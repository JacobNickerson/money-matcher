import sys
import os
import re
from time import sleep
import pandas as pd
from lightweight_charts.widgets import QtChart
from PyQt5.QtWidgets import ( 
    QApplication, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QSizePolicy, QMainWindow, QTableView, QStyledItemDelegate, QTabBar
)
from PyQt5.QtGui import (
    QFont, QIcon, QPixmap, QColor, QPainter, QPen
)
from PyQt5.QtCore import (
    Qt, QSize, QAbstractTableModel, QModelIndex
)

class SideBar(QWidget):
    def __init__(self):
        super().__init__()
        self.setFixedWidth(112)
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setStyleSheet("""
            SideBar {
                background-color: #101010;
                border-color: #363636;
                border-width: 1.5px;
                border-style: solid;
            }
        """)

        layout = QVBoxLayout()
        layout.setContentsMargins(10, 64, 10, 20)
        layout.setSpacing(30)
        layout.setAlignment(Qt.AlignTop | Qt.AlignHCenter)

        self.dashboard_btn = QPushButton()
        self.dashboard_btn.setIcon(QIcon("../../resources/images/dashboard.svg"))
        self.dashboard_btn.setIconSize(QSize(20, 20))
        self.bot_btn = QPushButton()
        self.bot_btn.setIcon(QIcon("../../resources/images/bot.svg"))
        self.bot_btn.setIconSize(QSize(20, 20))
        self.strat_btn = QPushButton()
        self.strat_btn.setIcon(QIcon("../../resources/images/strat.svg"))
        self.strat_btn.setIconSize(QSize(20, 20))
        self.stats_btn = QPushButton()
        self.stats_btn.setIcon(QIcon("../../resources/images/chart.svg"))
        self.stats_btn.setIconSize(QSize(20, 20))

        for btn in [self.dashboard_btn, self.bot_btn, self.strat_btn, self.stats_btn]:
            btn.setFixedSize(36, 36)
            btn.setStyleSheet("""
                QPushButton {
                    background-color: #101010;
                    border: none;
                    border-radius: 10px;
                    font-size: 20px;
                }
                QPushButton:hover {
                    background-color: #D9D9D9;
                }
                QPushButton:checked {
                    background-color: #D9D9D9;
                }
            """)
            btn.setCheckable(True)
            layout.addWidget(btn)

        self.dashboard_btn.setChecked(True)
        layout.addStretch()
        self.setLayout(layout)

class MarketEvents(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setFixedHeight(600)
        self.setFixedWidth(900)
        self.setStyleSheet("""
            MarketEvents {
                background-color: #101010;
                border-color: #363636;
                border-width: 1.5px;
                border-style: solid;
                border-radius: 16px;
            }
        """)
        layout = QVBoxLayout()
        layout.setContentsMargins(10, 10, 10, 10)

        widget = QWidget()
        widget.setLayout(layout)
        self.chart = QtChart(widget)
        layout.addWidget(self.chart.get_webview())
        self.setLayout(layout)

        test_data = pd.read_csv("../../resources/test_data/ohlc.csv") # TODO: replace with market data feed
        self.chart.set(test_data)
        self.chart.layout(background_color="#101010", text_color="#999999", font_family="Inter")
        self.chart.candle_style(up_color="#27AE60", down_color="#EB5757", wick_up_color="#27AE60", wick_down_color="#EB5757", border_visible=False)
        self.chart.grid(color="#363636")

class OrderBook(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(QSizePolicy.Preferred, QSizePolicy.Expanding)
        self.setStyleSheet("""
            OrderBook {
                background-color: #101010;
                border-color: #363636;
                border-width: 1.5px;
                border-style: solid;
                border-radius: 16px;
            }
        """)
        layout = QVBoxLayout()
        layout.setContentsMargins(10, 10, 10, 10)
        self.setLayout(layout)

        tabbar = QTabBar()
        tabbar.addTab("Order Book")
        tabbar.addTab("Trades")
        tabbar.setFont(QFont("Inter", 10, QFont.Medium))
        tabbar.setStyleSheet("""
            QTabBar::tab {
                background-color: #101010;
                color: #707070;
                padding: 8px 16px;
                border-bottom: 2px solid #1E1E1E;
            }
            QTabBar::tab:selected {
                color: #FDFDFD;
                border-bottom: 2px solid white;
            }
        """)
        layout.addWidget(tabbar)

class TradeHistoryModel(QAbstractTableModel):
    headers = [
        "Symbol", "Date", "Type", "Side",
        "Price", "Amount", "Filled", "Total", "Status", "Action"
    ]

    def __init__(self, rows):
        super().__init__()
        self.rows = rows

    def rowCount(self, parent=QModelIndex()):
        return len(self.rows)

    def columnCount(self, parent=QModelIndex()):
        return len(self.headers)

    def data(self, index, role):
        if not index.isValid():
            return None

        row = self.rows[index.row()]
        col = self.headers[index.column()]

        if role == Qt.DisplayRole:
            return row.get(col, "")
        
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font

        if role == Qt.ForegroundRole and col == "Side":
            return QColor("#27AE60") if row["Side"] == "Buy" else QColor("#EB5757")

        if role == Qt.TextAlignmentRole:
            return Qt.AlignLeft | Qt.AlignVCenter

        return None

    def headerData(self, section, orientation, role):
        if orientation == Qt.Horizontal:
            if role == Qt.DisplayRole:
                return self.headers[section]

        if role == Qt.TextAlignmentRole:
            return Qt.AlignLeft | Qt.AlignVCenter
        
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font
        
class CancelButtonDelegate(QStyledItemDelegate):
    def paint(self, painter, option, index):
        painter.save()

        rect = option.rect.adjusted(6, 6, -6, -6)
        painter.setRenderHint(QPainter.Antialiasing)

        painter.setBrush(QColor("#261719"))
        painter.setPen(Qt.NoPen)
        painter.drawRoundedRect(rect, 6, 6)

        painter.setPen(QColor("#FF5D61"))
        painter.drawText(rect, Qt.AlignCenter, "Cancel")

        painter.restore()

class TradeHistory(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)

        self.setStyleSheet("""
            TradeHistory {
                background-color: #101010;
                border: 1.5px solid #363636;
                border-radius: 16px;
            }
            QHeaderView::section {
                background-color: #101010;
                color: #707070;
                border: none;
                padding: 8px;
                font-size: 12px;
            }
            QTableView {
                background-color: #101010;
                border: none;
                color: white;
                gridline-color: transparent;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(12)

        btn_layout = QHBoxLayout()
        btn_layout.setSpacing(10)

        self.open_orders_btn = QPushButton("Open Orders")
        self.open_positions_btn = QPushButton("Open Positions")
        self.order_history_btn = QPushButton("Order History")

        for btn in (self.open_orders_btn, self.open_positions_btn, self.order_history_btn):
            btn.setCheckable(True)
            btn.setFixedSize(120, 32)
            btn.setFont(QFont("Inter", 10))
            btn.setStyleSheet("""
                QPushButton {
                    background-color: #101010;
                    color: #707070;
                    border-radius: 10px;
                    font-weight: 400;
                }
                QPushButton:checked {
                    background-color: #1e1e1e;
                    color: #eaeaea;
                }
            """)
            btn_layout.addWidget(btn)

        self.open_orders_btn.setChecked(True)
        btn_layout.addStretch()
        layout.addLayout(btn_layout)

        self.table = QTableView()
        self.table.verticalHeader().setVisible(False)
        self.table.verticalHeader().setDefaultSectionSize(44)
        self.table.setShowGrid(False)
        self.table.setSelectionMode(QTableView.NoSelection)
        self.table.setEditTriggers(QTableView.NoEditTriggers)
        self.table.horizontalHeader().setStretchLastSection(True)

        layout.addWidget(self.table)

        self.load_test_data()

    def load_test_data(self):
        rows = []
        for i in range(8):
            rows.append({
                "Symbol": "ETH/USDT",
                "Date": "Jan 26, 2025 5:30 PM",
                "Type": "Stop Limit" if i % 2 == 0 else "Limit",
                "Side": "Buy" if i % 3 else "Sell",
                "Price": "$0.90",
                "Amount": "8.5",
                "Filled": "10%",
                "Total": "715.00 USDT",
                "Status": "Open" if i % 2 else "Partial",
                "Action": ""
            })

        model = TradeHistoryModel(rows)
        self.table.setModel(model)

        delegate = CancelButtonDelegate(self.table)
        self.table.setItemDelegateForColumn(9, delegate)

        for row in range(model.rowCount()):
            side_index = model.index(row, 3)
            side = model.data(side_index, Qt.DisplayRole)
            self.table.model().rows[row]["Side"] = side

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