import sys
import os
from PyQt5.QtWidgets import ( 
    QApplication, QWidget, QGridLayout, QHBoxLayout,
    QSizePolicy, QStackedWidget, QVBoxLayout
)
from PyQt5.QtCore import (
    Qt, QTimer
)
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
    def __init__(self, fix_client=None):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QGridLayout(self)
        layout.setContentsMargins(20, 64, 20, 20)
        layout.setHorizontalSpacing(20)
        layout.setVerticalSpacing(20)

        self.rust_book = pyclient.PyOrderBook()
        self.mold_client = pyclient.PyMoldClient.start()

        self.market_events = MarketEvents()
        self.order_book = OrderBook()
        self.order_entry = OrderEntry(fix_client=fix_client)
        self.trade_history = TradeHistory()
        self.strategies = Strategies()

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

    def update_from_market_data(self):
        while True:
            event = self.mold_client.next_event()
            if event is None:
                break

            try:
                self.rust_book.process_event(event)
                self.market_events.handle_market_event(event)
            except Exception as e:
                print(f"Error processing event {event}: {e}")
                break

        self.order_book.refresh_order_book_display(self.rust_book)
        self.market_events.refresh_chart()
    
    def closeEvent(self, event):
        self.update_timer.stop()
        super().closeEvent(event)

class Bots(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(20)

        self.header = BotHeader()
        self.table = BotList()

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
        self.action_bar = ActionBar(self.editor)

        layout.addWidget(self.header)
        layout.addWidget(self.action_bar)
        layout.addWidget(self.editor, 1)

class Performance(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(0)

        self.header = PerfHeader()
        self.main = PerfMain()

        layout.addWidget(self.header)
        layout.addWidget(self.main, 1)

class EngineWindow(QWidget):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Money Matcher")
        self.resize(720, 512)
        self.fix_client = self.init_fix_client()
        self.initUI()

    def init_fix_client(self):
        try:
            fix_client = pyclient.PyFixClient.start(
                "127.0.0.1:34254",
                "CLIENT01",
                "ENGINE01"
            )
            return fix_client
        except Exception as e:
            print(f"Error initializing FIX Client: {e}")
            return None

    def initUI(self):
        main_layout = QHBoxLayout()
        main_layout.setContentsMargins(0,0,0,0)
        main_layout.setSpacing(0)

        # Sidebar
        self.sidebar = SideBar()
        self.sidebar.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.Expanding)

        self.stack = QStackedWidget()
        self.stack.setStyleSheet("background-color: #080808;")

        self.dashboard_page = Dashboard(fix_client=self.fix_client)
        self.bots_page = Bots()
        self.strat_page = Strats()
        self.perf_page = Performance()

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


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = EngineWindow()
    window.show()
    sys.exit(app.exec_())