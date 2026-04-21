import sys
import os
import sqlite3
from pathlib import Path
import time
from PyQt5.QtWidgets import ( 
    QApplication, QWidget, QGridLayout, QHBoxLayout, QPushButton,
    QSizePolicy, QStackedWidget, QVBoxLayout, QDialog, QMessageBox
)
from PyQt5.QtCore import (
    Qt, QTimer, pyqtSignal
)
from controllers.strategy_runner import StrategyRunner
from controllers.performance_tracker import PerformanceTracker
from controllers.simulated_execution_engine import SimulatedExecutionEngine
from widgets.sidebar import SideBar
from windows.dashboard import (
    MarketEvents, OrderBook, TradeHistory, OrderEntry, Strategies
)
from windows.strategies import (
    Header, ActionBar, CodeEditor
)
from windows.bots import (
    Header as BotHeader, BotList
)
from windows.performance import (
    Header as PerfHeader, Main as PerfMain
)

pyclient_dir = os.path.relpath('../../crates/pyclient')

if pyclient_dir not in sys.path:
    sys.path.append(pyclient_dir)

import pyclient

class Dashboard(QWidget):
    def __init__(self, fix_client=None, performance_tracker=None):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")
        self.strategy_runners = {}
        self.strategy_logs = {}
        self.strategy_sessions = {}

        layout = QGridLayout(self)
        layout.setContentsMargins(20, 64, 20, 20)
        layout.setHorizontalSpacing(20)
        layout.setVerticalSpacing(20)

        self.fix_client = fix_client
        self.rust_book = pyclient.PyOrderBook()
        self.mold_client = pyclient.PyMoldClient.start()
        self.execution_engine = SimulatedExecutionEngine()
        self.performance_tracker = performance_tracker

        self.market_events = MarketEvents()
        self.order_book = OrderBook()
        self.order_entry = OrderEntry(fix_client=self.fix_client)
        self.trade_history = TradeHistory(
            performance_tracker=self.performance_tracker, 
            strategy_sessions=self.strategy_sessions,
            order_entry=self.order_entry)
        self.strategies = Strategies(performance_tracker=self.performance_tracker)

        self.performance_tracker.performance_updated.connect(self.on_performance_update)
        self.on_performance_update(self.performance_tracker.get_account_summary())

        layout.addWidget(self.market_events, 0, 0)
        layout.addWidget(self.order_book, 0, 1)
        layout.addWidget(self.order_entry, 0, 2)

        layout.addWidget(self.trade_history, 1, 0, 1, 2)
        layout.addWidget(self.strategies, 1, 2)

        layout.setColumnStretch(0, 6)
        layout.setColumnStretch(1, 2)
        layout.setColumnStretch(2, 3)

        layout.setRowStretch(0, 5)
        layout.setRowStretch(1, 3)

        self.update_timer = QTimer(self)
        self.update_timer.timeout.connect(self.update_from_market_data)
        self.update_timer.start(250)

        self.strategy_timer = QTimer(self)
        self.strategy_timer.timeout.connect(self.on_strategy_timer)
        self.strategy_timer.start(1000)

    def set_fix_client(self, fix_client):
        self.fix_client = fix_client
        self.order_entry.fix_client = fix_client

    def start_bot(self, bot_config):
        bot_id = bot_config["id"]

        if bot_id in self.strategy_runners:
            return

        try:
            runner = self.strategy_sessions.get(bot_id)

            if runner is None:
                runner = StrategyRunner(
                    bot_config["strategy_file_path"],
                    bot_config,
                    order_entry=self.order_entry,
                    performance_tracker=self.performance_tracker,
                )
                runner.strategy_log.connect(
                    lambda msg, bot_id=bot_id: self.log_strategy_message(bot_id, msg)
                )
                runner.load_strategy()
                self.strategy_sessions[bot_id] = runner

            runner.start()
            self.strategy_runners[bot_id] = runner
            self.strategies.set_active_strategies(self.strategy_sessions)
            self.trade_history.set_strategy_sessions(self.strategy_sessions)
            self.refresh_strategy_panel()

        except Exception as e:
            print(f"Error starting bot {bot_id}: {e}")

    def stop_bot(self, bot_id):
        runner = self.strategy_runners.get(bot_id)
        if runner is None:
            return
        
        try:
            runner.stop()
        except Exception as e:
            print(f"Error stopping bot {bot_id}: {e}")
        finally:
            del self.strategy_runners[bot_id]
            self.strategies.set_active_strategies(self.strategy_sessions)
            self.trade_history.set_strategy_sessions(self.strategy_sessions)
            self.refresh_strategy_panel()

    def update_from_market_data(self):
        processed_event = False

        while True:
            event = self.mold_client.next_event()
            if event is None:
                break

            try:
                self.rust_book.process_event(event)
                self.market_events.handle_market_event(event)

                for runner in self.strategy_runners.values():
                    runner.on_market_event(event, self.rust_book)

                processed_event = True
            except Exception as e:
                print(f"Error processing event {event}: {e}")
                break

        if processed_event:
            self.order_book.refresh_order_book_display(self.rust_book)
            self.market_events.refresh_chart()
            self.trade_history.refresh_data()
            self.order_entry.set_book(self.rust_book.best_bid(), self.rust_book.best_ask())

        self.refresh_strategy_panel()

        now = time.time()
        for runner in self.strategy_runners.values():
            try:
                self.execution_engine.process_runner(runner, now)
            except Exception as e:
                print(f"Error processing runner fills: {e}")

        try:
            self.execution_engine.process_manual_orders(
                self.order_entry,
                self.rust_book,
                now,
                on_fill=self.on_manual_fill
            )
        except Exception as e:
            print(f"Error processing manual fills: {e}")

    def on_manual_fill(self, fill):
        print(f"Manual fill: {fill.side} {fill.qty} @ {fill.price}")

    def on_performance_update(self, summary):
        quote_balance = float(summary.get("cash_balance", 0.0))
        base_balance = float(summary.get("base_balance", 0.0))
        self.order_entry.set_balances(quote_balance, base_balance)

    def on_strategy_timer(self):
        now = time.time()
        for runner in self.strategy_runners.values():
            try:
                runner.on_timer(now)
            except Exception as e:
                print(f"Error in strategy timer: {e}")
    
    def closeEvent(self, event):
        self.update_timer.stop()
        for bot_id, runner in self.strategy_runners.items():
            try:
                runner.stop()
            except Exception as e:
                print(f"Error stopping bot {bot_id}: {e}")
        super().closeEvent(event)

    def refresh_strategy_panel(self):
        self.strategies.set_active_strategies(self.strategy_sessions)

        bot_id = self.strategies.current_bot_id
        runner = self.strategy_sessions.get(bot_id)
        if runner is None:
            self.strategies.clear_stats()
            self.strategies.clear_logs()
            return

        stats = runner.get_stats()
        self.strategies.update_strategy_stats(stats)
        self.strategies.set_logs(self.strategy_logs.get(bot_id, []))

    def log_strategy_message(self, bot_id, msg):
        if bot_id not in self.strategy_logs:
            self.strategy_logs[bot_id] = []

        self.strategy_logs[bot_id].append(msg)
        self.strategy_logs[bot_id] = self.strategy_logs[bot_id][-100:]
        self.strategies.add_strategy_log(bot_id, msg)

class Bots(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(20)

        self.table = BotList()
        self.header = BotHeader(self.table)

        layout.addWidget(self.header)
        layout.addWidget(self.table, 1)

class Strats(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(0)

        self.editor = CodeEditor()
        self.header = Header(self.editor)
        self.action_bar = ActionBar(self.editor, self.header)
        self.header.strategyChanged.connect(self.action_bar.setCurrentStrategy)
        self.header.loadSymbols()

        layout.addWidget(self.header)
        layout.addWidget(self.action_bar)
        layout.addWidget(self.editor, 1)

class Performance(QWidget):
    def __init__(self, performance_tracker):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(0)

        self.main = PerfMain(performance_tracker=performance_tracker)

        layout.addWidget(self.main, 1)

class FixInitModal(QDialog):
    fix_initialized = pyqtSignal(object)

    def __init__(self, parent=None):
        super().__init__(parent)

        self.fix_client = None

        self.setModal(True)
        self.setFixedSize(320, 160)
        self.setWindowTitle("Connect")

        self.setStyleSheet("""
            QDialog {
                background-color: #101010;
                border: 1px solid #363636;
                border-radius: 12px;
            }
            QPushButton {
                background-color: #FFFFFF;
                color: #080808;
                border-radius: 8px;
                padding: 10px;
                font-weight: 600;
            }
            QPushButton:hover {
                background-color: #D9D9D9;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)

        self.connect_btn = QPushButton("Connect")
        self.connect_btn.setMinimumHeight(40)
        self.connect_btn.clicked.connect(self.handle_connect)

        layout.addStretch()
        layout.addWidget(self.connect_btn)
        layout.addStretch()

    def handle_connect(self):
        try:
            self.fix_client = pyclient.PyFixClient.start(
                "127.0.0.1:34254",
                "CLIENT01",
                "ENGINE01"
            )
            self.fix_initialized.emit(self.fix_client)
            self.accept()
        except Exception as e:
            QMessageBox.warning(self, "Error", str(e))

class EngineWindow(QWidget):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Money Matcher")
        self.resize(720, 512)
        self.fix_client = None
        self.account_balance = self.initAccountDatabase(default_balance=10000.0)
        self.performance_tracker = PerformanceTracker(starting_balance=self.account_balance)
        self.initUI()
        #self.show_fix_modal()
        QTimer.singleShot(0, self.show_fix_modal)

    def initUI(self):
        main_layout = QHBoxLayout()
        main_layout.setContentsMargins(0,0,0,0)
        main_layout.setSpacing(0)

        # Sidebar
        self.sidebar = SideBar()
        self.sidebar.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.Expanding)

        self.stack = QStackedWidget()
        self.stack.setStyleSheet("background-color: #080808;")

        self.dashboard_page = Dashboard(fix_client=self.fix_client, performance_tracker=self.performance_tracker)
        self.bots_page = Bots()
        self.strat_page = Strats()
        self.perf_page = Performance(performance_tracker=self.performance_tracker)

        bot_list = self.bots_page.table
        bot_list.bot_started.connect(self.dashboard_page.start_bot)
        bot_list.bot_stopped.connect(self.dashboard_page.stop_bot)

        self.stack.addWidget(self.dashboard_page)
        self.stack.addWidget(self.bots_page)
        self.stack.addWidget(self.strat_page)
        self.stack.addWidget(self.perf_page)

        main_layout.addWidget(self.sidebar)
        main_layout.addWidget(self.stack)

        self.setLayout(main_layout)

        self.sidebar.dashboard_btn.clicked.connect(
            lambda: self.stack.setCurrentIndex(0)
        )
        self.sidebar.bot_btn.clicked.connect(
            lambda: self.stack.setCurrentIndex(1)
        )
        self.sidebar.strat_btn.clicked.connect(
            lambda: self.stack.setCurrentIndex(2)
        )
        self.sidebar.chart_btn.clicked.connect(
            lambda: self.stack.setCurrentIndex(3)
        )

    def show_fix_modal(self):
        modal = FixInitModal(self)
        modal.fix_initialized.connect(self.on_fix_connected)
        modal.exec()

    def on_fix_connected(self, fix_client):
        self.fix_client = fix_client
        self.dashboard_page.set_fix_client(fix_client)

    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[1]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"

    def initAccountDatabase(self, default_balance=10000.0):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS account (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                initial_balance REAL NOT NULL,
                cash_balance REAL NOT NULL,
                created_at REAL NOT NULL,
                updated_at REAL NOT NULL
            )
        """)

        cursor.execute("SELECT cash_balance FROM account WHERE id = 1")
        row = cursor.fetchone()

        if row is None:
            now = time.time()

            cursor.execute("""
                INSERT INTO account (id, initial_balance, cash_balance, created_at, updated_at)
                VALUES (1, ?, ?, ?, ?)
            """, (default_balance, default_balance, now, now))

            conn.commit()
            balance = default_balance
        else:
            balance = float(row[0])

        conn.close()
        return balance

if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = EngineWindow()
    window.show()
    sys.exit(app.exec_())