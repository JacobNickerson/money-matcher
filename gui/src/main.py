import sys
from PyQt5.QtWidgets import ( 
    QApplication, QWidget, QVBoxLayout, QHBoxLayout,
    QSizePolicy
)

from PyQt5.QtCore import (
    Qt
)
from widgets.sidebar import SideBar
from windows.dashboard import (
    MarketEvents, OrderBook, TradeHistory, OrderEntry
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
        content_layout = QVBoxLayout(content_widget)
        content_layout.setContentsMargins(0,0,0,0)
        content_layout.setContentsMargins(20, 64, 20, 20)
        content_layout.setSpacing(20)
        content_layout.setAlignment(Qt.AlignTop | Qt.AlignLeft)

        row_layout_1 = QHBoxLayout()
        row_layout_1.setSpacing(20)
        row_layout_2 = QHBoxLayout()
        row_layout_2.setSpacing(20)

        # MarketEvents
        self.market_events = MarketEvents()
        row_layout_1.addWidget(self.market_events)

        # OrderBook
        self.order_book = OrderBook()
        row_layout_1.addWidget(self.order_book)

        # OrderEntry
        self.order_entry = OrderEntry()
        row_layout_1.addWidget(self.order_entry)

        # TradeHistory
        self.trade_history = TradeHistory()
        row_layout_2.addWidget(self.trade_history)

        content_layout.addLayout(row_layout_1)
        content_layout.addLayout(row_layout_2, 2)
        content_layout.addStretch()

        main_layout.addWidget(self.sidebar)
        main_layout.addWidget(content_widget)

        self.setLayout(main_layout)


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = EngineWindow()
    window.show()
    sys.exit(app.exec_())