import pandas as pd
from lightweight_charts.widgets import QtChart
from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton,
    QSizePolicy, QTableView, QStyledItemDelegate, QTabBar, QHeaderView
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter
)
from PyQt5.QtCore import (
    Qt, QRect
)

import models.order_book_model as order_book_model
import models.trade_history_model as trade_history_model

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

class OrderBookTableView(QTableView):
    def __init__(self):
        super().__init__()
        self.verticalHeader().setVisible(False)
        self.setShowGrid(False)
        self.setSelectionMode(QTableView.NoSelection)
        self.setEditTriggers(QTableView.NoEditTriggers)
        self.setFocusPolicy(Qt.NoFocus)
        self.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)
        self.verticalHeader().setDefaultSectionSize(32)
        
        self.ask_color = QColor("#251717")
        self.bid_color = QColor("#17291B")
        self.mid_color = QColor("#1E1E1E")
    
    def paintEvent(self, event):
        painter = QPainter(self.viewport())
        painter.setRenderHint(QPainter.Antialiasing, True)
        
        model = self.model()
        if model:
            max_amount = model.max_amount()
            
            first_row = self.rowAt(0)
            last_row = self.rowAt(self.viewport().height())
            if last_row == -1:
                last_row = model.rowCount() - 1
            
            for row in range(first_row, last_row + 1):
                side, price, amount, total = model.row_info(row)

                left_rect = self.visualRect(model.index(row, 0))
                right_rect = self.visualRect(model.index(row, model.columnCount() - 1))
                row_rect = left_rect.united(right_rect)
                row_rect = row_rect.adjusted(0, 3, 0, -3)
                painter.setPen(Qt.NoPen)
                
                if side == "mid":
                    mid_width = int(row_rect.width())
                    bar_rect = QRect(
                        row_rect.center().x() - mid_width // 2,
                        row_rect.top(),
                        mid_width,
                        row_rect.height()
                    )
                    color = self.mid_color

                elif amount > 0:

                    if side == "mid":
                        ratio = 1
                    else:
                        ratio = amount / max_amount
                    
                    bar_width = int(row_rect.width() * ratio)
                    bar_rect = QRect(
                        row_rect.right() - bar_width,
                        row_rect.top(),
                        bar_width,
                        row_rect.height()
                    )
                    
                    if side == "ask":
                        color = self.ask_color
                    else:
                        color = self.bid_color

                painter.setBrush(color)
                painter.drawRoundedRect(bar_rect, 4, 4)
        
        super().paintEvent(event)

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
        layout.setContentsMargins(16, 16, 16, 16)
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

        self.table = OrderBookTableView()
        self.table.setStyleSheet("""
            QTableView {
                background-color: #101010;
                color: white;
                font-size: 12px;
                gridline-color: transparent;
                border: none;
            }
            QHeaderView::section {
                background-color: #101010;
                color: #999999;
                padding: 4px;
                border: none;
            }                      
        """)
        
        self.loadTestData()

        scroll_bar_style = """
            QScrollBar:vertical {
                background-color: #101010;
                width: 8px;
                margin: 0px;
                border: none;
            }
            QScrollBar::handle:vertical {
                background-color: #363636;
                min-height: 20px;
                border-radius: 4px;
            }
            QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {
                background-color: #101010;
            }
        """
        self.table.verticalScrollBar().setStyleSheet(scroll_bar_style)

        self.layout().addWidget(self.table)

    def loadTestData(self):
        asks = [(100.50 + i, 1.5 + i*0.1, (100.50 + i) * (1.5 + i*0.1)) for i in range(10)]
        bids = [(100.00 - i, 2.0 + i*0.2, (100.00 - i) * (2.0 + i*0.2)) for i in range(10)]

        model = order_book_model.OrderBookModel(asks, bids)
        self.table.setModel(model)
        self.table.resizeColumnsToContents()

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
        self.table.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)
        self.table.setFocusPolicy(Qt.NoFocus)

        scroll_bar_style = """
            QScrollBar:vertical {
                background-color: #101010;
                width: 8px;
                margin: 0px;
                border: none;
            }
            QScrollBar::handle:vertical {
                background-color: #363636;
                min-height: 20px;
                border-radius: 4px;
            }
            QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
                height: 0px;
            }
            QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {
                background-color: #101010;
            }
        """
        self.table.verticalScrollBar().setStyleSheet(scroll_bar_style)

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

        model = trade_history_model.TradeHistoryModel(rows)
        self.table.setModel(model)

        delegate = CancelButtonDelegate(self.table)
        self.table.setItemDelegateForColumn(9, delegate)

        for row in range(model.rowCount()):
            side_index = model.index(row, 3)
            side = model.data(side_index, Qt.DisplayRole)
            self.table.model().rows[row]["Side"] = side