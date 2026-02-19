import sys
from PyQt5.QtWidgets import ( 
    QApplication, QWidget, QGridLayout, QHBoxLayout,
    QSizePolicy, QStackedWidget, QVBoxLayout
)

from PyQt5.QtCore import (
    Qt
)
from widgets.sidebar import SideBar
from windows.dashboard import (
    MarketEvents, OrderBook, TradeHistory, OrderEntry, Strategies
)

from windows.strategies import (
    Header, ActionBar, CodeEditor
)

class Dashboard(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QGridLayout(self)
        layout.setContentsMargins(20, 64, 20, 20)
        layout.setHorizontalSpacing(20)
        layout.setVerticalSpacing(20)

        self.market_events = MarketEvents()
        self.order_book = OrderBook()
        self.order_entry = OrderEntry()
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

class Bots(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

class Strats(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background-color: #080808;")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(0)

        self.header = Header()
        self.action_bar = ActionBar()
        self.editor = CodeEditor()

        layout.addWidget(self.header)
        layout.addWidget(self.action_bar)
        layout.addWidget(self.editor, 1)

class EngineWindow(QWidget):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Money Matcher")
        self.resize(720, 512)
        self.initUI()

    def initUI(self):
        main_layout = QHBoxLayout()
        main_layout.setContentsMargins(0,0,0,0)
        main_layout.setSpacing(0)

        # Sidebar
        self.sidebar = SideBar()
        self.sidebar.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.Expanding)

        self.stack = QStackedWidget()
        self.stack.setStyleSheet("background-color: #080808;")

        self.dashboard_page = Dashboard()
        self.bots_page = Bots()
        self.strat_page = Strats()

        self.stack.addWidget(self.dashboard_page)
        self.stack.addWidget(self.bots_page)
        self.stack.addWidget(self.strat_page)

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


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = EngineWindow()
    window.show()
    sys.exit(app.exec_())