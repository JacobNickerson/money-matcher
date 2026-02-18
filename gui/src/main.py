import sys
from PyQt5.QtWidgets import ( 
    QApplication, QWidget, QGridLayout, QHBoxLayout,
    QSizePolicy
)

from PyQt5.QtCore import (
    Qt
)
from widgets.sidebar import SideBar
from windows.dashboard import (
    MarketEvents, OrderBook, TradeHistory, OrderEntry, Strategies
)

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
        self.sidebar.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Expanding)

        # Content Area
        content_widget = QWidget()
        content_widget.setStyleSheet("background-color: #080808;")
        content_layout = QGridLayout(content_widget)
        content_layout.setContentsMargins(20, 64, 20, 20)
        content_layout.setHorizontalSpacing(20)
        content_layout.setVerticalSpacing(20)

        self.market_events = MarketEvents()
        self.order_book = OrderBook()
        self.order_entry = OrderEntry()
        self.trade_history = TradeHistory()
        self.strategies = Strategies()

        content_layout.addWidget(self.market_events, 0, 0)
        content_layout.addWidget(self.order_book, 0, 1)
        content_layout.addWidget(self.order_entry, 0, 2)

        content_layout.addWidget(self.trade_history, 1, 0, 1, 2)
        content_layout.addWidget(self.strategies, 1, 2)

        content_layout.setColumnStretch(0, 6)  # MarketEvents
        content_layout.setColumnStretch(1, 2)  # OrderBook
        content_layout.setColumnStretch(2, 3)  # OrderEntry

        content_layout.setRowStretch(0, 5)
        content_layout.setRowStretch(1, 3)

        main_layout.addWidget(self.sidebar)
        main_layout.addWidget(content_widget)

        self.setLayout(main_layout)


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = EngineWindow()
    window.show()
    sys.exit(app.exec_())