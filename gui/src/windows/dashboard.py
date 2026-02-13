import pandas as pd
from lightweight_charts.widgets import QtChart
from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QLineEdit,
    QSizePolicy, QTableView, QStyledItemDelegate, QTabBar, QHeaderView,
    QFrame, QGridLayout
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
                border-width: 1px;
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
        symbol = "SOL/USD" # TODO: add symbol selector and update chart based on selection
        self.chart.set(test_data)
        self.chart.layout(background_color="#101010", text_color="#999999", font_family="Inter")
        self.chart.candle_style(up_color="#27AE60", down_color="#EB5757", wick_up_color="#27AE60", wick_down_color="#EB5757", border_visible=False)
        self.chart.grid(color="#080808")
        self.chart.legend(visible=True, font_size=14, font_family="Inter", color="#999999", color_based_on_candle=True, text=symbol, lines=False)
        self.chart.price_scale(border_visible=False)
        self.chart.time_scale(border_visible=False)

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
                border-width: 1px;
                border-style: solid;
                border-radius: 16px;
            }
        """)
        layout = QVBoxLayout()
        layout.setContentsMargins(20, 20, 20, 20)
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
                color: white;
                border-bottom: 2px solid white;
            }
        """)
        tabbar.setCursor(Qt.PointingHandCursor)
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

class OrderEntry(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(QSizePolicy.Preferred, QSizePolicy.Expanding)

        self.setStyleSheet("""
            OrderEntry {
                background-color: #101010;
                border-width: 1px;
                border-style: solid;
                border-color: #363636;
                border-radius: 16px;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(18)

        tabbar = QTabBar()
        tabbar.addTab("Spot")
        tabbar.addTab("Bots")
        tabbar.setFont(QFont("Inter", 10, QFont.Medium))
        tabbar.setStyleSheet("""
            QTabBar::tab {
                background-color: #101010;
                color: #707070;
                padding: 8px 16px;
                border-bottom: 2px solid #1E1E1E;
            }
            QTabBar::tab:selected {
                color: white;
                border-bottom: 2px solid white;
            }
        """)
        tabbar.setCursor(Qt.PointingHandCursor)
        layout.addWidget(tabbar)

        btn_container = QWidget()
        btn_container.setAttribute(Qt.WA_StyledBackground, True)
        btn_container.setStyleSheet("""
            QWidget {
                border: 1.5px;
                border-style: solid;
                border-color: #1E1E1E;
                border-radius: 12px;
                background-color: #101010;
            }
        """)

        btn_layout = QHBoxLayout(btn_container)
        btn_layout.setContentsMargins(6, 6, 6, 6)
        btn_layout.setSpacing(0)

        self.buy_btn = QPushButton("Buy")
        self.sell_btn = QPushButton("Sell")

        for btn in (self.buy_btn, self.sell_btn):
            btn.setCheckable(True)
            btn.setFixedHeight(32)
            btn.setFont(QFont("Inter", 10))
            btn.setStyleSheet("""
                QPushButton {
                    background-color: #101010;
                    color: #707070;
                    border-radius: 8px;
                    border: none;
                }
                QPushButton:checked {
                    background-color: #1E1E1E;
                    color: white;
                }
            """)
            btn_layout.addWidget(btn)

        self.buy_btn.setChecked(True)
        self.buy_btn.setAutoExclusive(True)
        self.sell_btn.setAutoExclusive(True)
        self.buy_btn.setCursor(Qt.PointingHandCursor)
        self.sell_btn.setCursor(Qt.PointingHandCursor)
        layout.addWidget(btn_container)

        type_container = QWidget()
        type_container.setAttribute(Qt.WA_StyledBackground, True)
        type_container.setStyleSheet("""
            QWidget {
                background-color: #101010;
            }
        """)
        type_layout = QHBoxLayout(type_container)
        type_layout.setContentsMargins(0, 0, 0, 0)
        type_layout.setSpacing(16)

        self.market_lbl = QLabel("Market")
        self.market_lbl.setFont(QFont("Inter", 10))
        self.market_lbl.setStyleSheet("""
            color: #999999;
            background-color: #101010;
        """)

        self.limit_btn = QPushButton("Limit")
        self.tpsl_btn = QPushButton("Tp/SL")

        for btn in (self.limit_btn, self.tpsl_btn):
            btn.setCheckable(True)
            btn.setFont(QFont("Inter", 10))
            btn.setStyleSheet("""
                QPushButton {
                    background: #101010;
                    color: #707070;
                    border: none;
                    padding-bottom: 2px;
                }
                QPushButton:checked {
                    color: white;
                    border-bottom: 2px solid white;
                }
            """)
            type_layout.addWidget(btn)

        self.limit_btn.setChecked(True)
        self.limit_btn.setCursor(Qt.PointingHandCursor)
        self.limit_btn.setAutoExclusive(True)
        self.tpsl_btn.setCursor(Qt.PointingHandCursor)
        self.tpsl_btn.setAutoExclusive(True)

        type_layout.addStretch()
        type_layout.insertWidget(0, self.market_lbl)
        layout.addWidget(type_container)

        layout.addWidget(self.input_field("Price", "0.00"))
        layout.addWidget(self.input_field("Amount", "0.0000"))
        layout.addWidget(self.input_field("Total", "0.000"))

        layout.addWidget(self.table())

        layout.addStretch()

        self.submit_btn = QPushButton()
        self.submit_btn.setFixedHeight(44)
        self.submit_btn.setFont(QFont("Inter", 11, QFont.Bold))
        self.submit_btn.setCursor(Qt.PointingHandCursor)

        self.update_submit_button()
        self.buy_btn.toggled.connect(self.update_submit_button)
        self.sell_btn.toggled.connect(self.update_submit_button)

        layout.addWidget(self.submit_btn)

    def input_field(self, label_text, placeholder):
        container = QWidget()
        container.setAttribute(Qt.WA_StyledBackground, True)
        container.setStyleSheet("""
            QWidget {
                background-color: #101010;
            }
        """)
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(6)

        label = QLabel(label_text)
        label.setFont(QFont("Inter", 9))
        label.setStyleSheet("color: #999999;")

        field = QLineEdit()
        field.setPlaceholderText(placeholder)
        field.setFont(QFont("Inter", 10))
        field.setFixedHeight(36)
        field.setStyleSheet("""
            QLineEdit {
                background-color: #1D1D1D;
                border: none;
                border-radius: 8px;
                padding-right: 10px;
                color: white;
            }
        """)
        field.setAlignment(Qt.AlignRight | Qt.AlignVCenter)

        field_layout = QHBoxLayout()
        field_layout.setContentsMargins(0, 0, 0, 0)
        field_layout.addWidget(field)

        layout.addWidget(label)
        layout.addLayout(field_layout)

        return container

    def table(self, available_val="0.000", max_buy_val="0.000"):
        container = QWidget()
        container.setAttribute(Qt.WA_StyledBackground, True)
        container.setStyleSheet("""
            QWidget {
                background-color: #101010;
                border: 1px solid #2A2A2A;
                border-radius: 10px;
            }
            QLabel {
                font-family: Inter;
            }
        """)

        grid = QGridLayout(container)
        grid.setContentsMargins(2, 2, 2, 2)
        grid.setHorizontalSpacing(0)
        grid.setVerticalSpacing(0)
        grid.setColumnStretch(0, 1)
        grid.setColumnStretch(1, 0)
        grid.setColumnStretch(2, 1)

        available_title = QLabel("Available")
        available_title.setStyleSheet("""
            color: white; 
            font-size: 10pt;
            border: none;
            padding: 8px;
        """)

        max_buy_title = QLabel("Max Buy")
        max_buy_title.setStyleSheet("""
            color: white; 
            font-size: 10pt;
            border: none;
            padding: 8px;
        """)

        available_value = QLabel(available_val)
        available_value.setStyleSheet("""
            color: #707070; 
            font-size: 9pt;
            padding: 8px;
            border: none;
        """)

        max_buy_value = QLabel(max_buy_val)
        max_buy_value.setStyleSheet("""
            color: #707070; 
            font-size: 9pt;
            padding: 8px;
            border: none;
        """)

        grid.addWidget(available_title, 0, 0)
        grid.addWidget(max_buy_title, 0, 2)

        grid.addWidget(available_value, 2, 0)
        grid.addWidget(max_buy_value, 2, 2)

        h_divider = QFrame()
        h_divider.setFixedHeight(1)
        h_divider.setFrameShape(QFrame.HLine)
        h_divider.setStyleSheet("color: #1D1D1D;")
        grid.addWidget(h_divider, 1, 0, 1, 3)

        v_divider = QFrame()
        v_divider.setFixedWidth(1)
        v_divider.setFrameShape(QFrame.VLine)
        v_divider.setStyleSheet("color: #1D1D1D;")
        grid.addWidget(v_divider, 0, 1, 3, 1)

        return container

    def update_submit_button(self):
        if self.buy_btn.isChecked():
            self.submit_btn.setText("Buy")
            self.submit_btn.setStyleSheet("""
                QPushButton {
                    background-color: #27AE60;
                    color: white;
                    border-radius: 8px;
                    border: none;
                }
                QPushButton:hover {
                    background-color: #2ECC71;
                }
                QPushButton:pressed {
                    background-color: #24914D;
                }
            """)
        else:
            self.submit_btn.setText("Sell")
            self.submit_btn.setStyleSheet("""
                QPushButton {
                    background-color: #EB5757;
                    color: white;
                    border-radius: 8px;
                }
                QPushButton:hover {
                    background-color: #FF6E6E;
                }
                QPushButton:pressed {
                    background-color: #C44B4B;
                }
            """)

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
                border-width: 1px;
                border-style: solid;
                border-color: #363636;
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
        self.open_orders_btn.setAutoExclusive(True)
        self.open_positions_btn.setAutoExclusive(True)
        self.order_history_btn.setAutoExclusive(True)
        self.open_orders_btn.setCursor(Qt.PointingHandCursor)
        self.open_positions_btn.setCursor(Qt.PointingHandCursor)
        self.order_history_btn.setCursor(Qt.PointingHandCursor)
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