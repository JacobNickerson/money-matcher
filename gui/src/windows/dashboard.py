import os
import sys
import threading
import time
import pandas as pd
from decimal import Decimal
from lightweight_charts.widgets import QtChart
from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QLineEdit,
    QSizePolicy, QTableView, QStyledItemDelegate, QTabBar, QHeaderView,
    QFrame, QGridLayout, QComboBox, QScrollArea
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter
)
from PyQt5.QtCore import (
    Qt, QRect, QTime
)

import models.order_book_model as order_book_model
import models.trade_history_model as trade_history_model
import controllers.candlestick as candle

pyclient_dir = os.path.relpath('../../crates/pyclient')

if pyclient_dir not in sys.path:
    sys.path.append(pyclient_dir)

import pyclient

class MarketEvents(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(
            QSizePolicy.Policy.Expanding,
            QSizePolicy.Policy.Expanding
        )
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

        self.chart_controller = candle.CandlestickController(chart=self.chart)

    def handle_market_event(self, event):
        self.chart_controller.handle_market_event(event)

    def refresh_chart(self):
        self.chart_controller.refresh_chart()

class OrderBookTableView(QTableView):
    def __init__(self):
        super().__init__()
        self.verticalHeader().setVisible(False)
        self.setShowGrid(False)
        self.setSelectionMode(QTableView.NoSelection)
        self.setEditTriggers(QTableView.NoEditTriggers)
        self.setFocusPolicy(Qt.NoFocus)
        self.horizontalHeader().setSectionResizeMode(QHeaderView.Stretch)
        self.verticalHeader().setSectionResizeMode(QHeaderView.Stretch)
        
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
        self.setMinimumWidth(320)
        self.setMaximumWidth(350)
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
        self.table.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
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

        self.layout().addWidget(self.table)

    def refresh_order_book_display(self, rust_book):
        try:
            raw_asks = rust_book.get_top_levels(pyclient.PyOrderSide.Ask, 7)
            raw_bids = rust_book.get_top_levels(pyclient.PyOrderSide.Bid, 7)
            asks = [(price, qty, price * qty) for price, qty in raw_asks]
            bids = [(price, qty, price * qty) for price, qty in raw_bids]
            spread = rust_book.spread()
            mid_price = rust_book.mid_price()
            model = order_book_model.OrderBookModel(asks, bids, spread, mid_price)
            self.table.setModel(model)
            #self.table.resizeColumnsToContents()
        except Exception as e:
            print(f"Error updating order book: {e}")

class OrderIdGenerator:
    def __init__(self):
        self.last = 0
        self.lock = threading.Lock()

    def next(self):
        with self.lock:
            now = time.time_ns()
            if now <= self.last:
                now = self.last + 1
            self.last = now
            return now
    
class OrderEntry(QWidget):
    def __init__(self, fix_client=None):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(QSizePolicy.Preferred, QSizePolicy.Expanding)
        self.setMinimumWidth(320)
        self.fix_client = fix_client
        self.orderid_gen = OrderIdGenerator()

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

        self.market_lbl = QLabel("Order Type")
        self.market_lbl.setFont(QFont("Inter", 10))
        self.market_lbl.setStyleSheet("""
            color: #999999;
            background-color: #101010;
        """)

        self.limit_btn = QPushButton("Limit")
        self.tpsl_btn = QPushButton("Market")

        for btn in (self.limit_btn, self.tpsl_btn):
            btn.setCheckable(True)
            btn.setFont(QFont("Inter", 10))
            btn.setCursor(Qt.PointingHandCursor)
            btn.setAutoExclusive(True)
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

        type_layout.addStretch()
        type_layout.insertWidget(0, self.market_lbl)
        layout.addWidget(type_container)

        price_widget, self.price_input = self.input_field("Price", "0.00")
        amount_widget, self.amount_input = self.input_field("Amount", "0.0000")
        self.price_input.textChanged.connect(self.update_total)
        self.amount_input.textChanged.connect(self.update_total)

        total_widget, self.total_input = self.input_field("Total", "0.000")
        self.total_input.setReadOnly(True)
        self.total_input.setFocusPolicy(Qt.NoFocus)

        layout.addWidget(price_widget)
        layout.addWidget(amount_widget)
        layout.addWidget(total_widget)

        layout.addWidget(self.table())

        self.submit_btn = QPushButton()
        self.submit_btn.setMinimumHeight(32)
        self.submit_btn.setMaximumHeight(44)
        self.submit_btn.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        self.submit_btn.setFont(QFont("Inter", 11, QFont.Bold))
        self.submit_btn.setCursor(Qt.PointingHandCursor)

        self.update_submit_button()
        self.buy_btn.toggled.connect(self.update_submit_button)
        self.sell_btn.toggled.connect(self.update_submit_button)
        self.submit_btn.clicked.connect(self.submit_order)

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
        layout.setSpacing(4)

        label = QLabel(label_text)
        label.setFont(QFont("Inter", 9))
        label.setStyleSheet("color: #999999;")

        field = QLineEdit()
        field.setPlaceholderText(placeholder)
        field.setFont(QFont("Inter", 10))
        field.setMinimumHeight(30)
        field.setMaximumHeight(36)

        field.setStyleSheet("""
            QLineEdit {
                background-color: #1D1D1D;
                border: none;
                border-radius: 6px;
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

        return container, field

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
    
    def update_total(self):
        try:
            price = float(self.price_input.text())
            amount = float(self.amount_input.text())
            total = price * amount

            self.total_input.setText(f"{total:.5f}")
        except ValueError:
            self.total_input.setText("")

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
    
    def submit_order(self, bot_order=False, _side=None, _price=None, _qty=None):
        try:
            if bot_order:
                side = pyclient.PyOrderSide.Bid if _side.upper() == "BUY" else pyclient.PyOrderSide.Ask
                price = int(Decimal(_price) * Decimal("1e4"))
                qty = int(Decimal(_qty) * Decimal("1e4"))
            else:
                side = pyclient.PyOrderSide.Bid if self.buy_btn.isChecked() else pyclient.PyOrderSide.Ask
                price = int(Decimal(self.price_input.text()) * Decimal("1e4"))
                qty = int(Decimal(self.amount_input.text()) * Decimal("1e4"))

            order_type = pyclient.PyOrderType.limit(qty, price) if self.limit_btn.isChecked() else pyclient.PyOrderType.market(qty)
            order = pyclient.PyOrder(order_id=self.orderid_gen.next(), side=side, timestamp=int(time.time() * 1000), kind=order_type)
            
            if self.fix_client:
                self.fix_client.send_message(order)
        except Exception as e:
            print(f"Error sending order: {e}")

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
            btn.setCursor(Qt.PointingHandCursor)
            btn.setAutoExclusive(True)
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
        self.table.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)

        layout.addWidget(self.table)

        #self.load_test_data()

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

class LogCard(QFrame):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("""
            QFrame {
                background-color: #080808;
                border: 1px solid #363636;
                border-radius: 12px;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(4, 0, 4, 0)

        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)

        self.container = QWidget()
        self.log_layout = QVBoxLayout(self.container)
        self.log_layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.log_layout.setSpacing(6)

        self.scroll.setWidget(self.container)
        layout.addWidget(self.scroll)

    def add_log(self, text, color="white", time=""):
        row = QWidget()
        row_layout = QHBoxLayout(row)
        row_layout.setContentsMargins(0, 0, 0, 0)

        label = QLabel(text)
        label.setFont(QFont("Inter", 10, QFont.Normal))
        label.setStyleSheet(f"color: {color}; border: none;")
        label.setWordWrap(True)

        time_label = QLabel(time)
        time_label.setFont(QFont("Inter", 10, QFont.Normal))
        time_label.setStyleSheet("color: #999999; border: none;")
        time_label.setAlignment(Qt.AlignmentFlag.AlignRight)

        row_layout.addWidget(label, stretch=1)
        row_layout.addWidget(time_label)

        self.log_layout.addWidget(row)

class Strategies(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        self.setStyleSheet("""
            Strategies {
                background-color: #101010;
                border-width: 1px;
                border-style: solid;
                border-color: #363636;
                border-radius: 16px;
            }
        """)
        self.strategy_map = {}
        self.current_bot_id = None

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 16, 16, 16)
        layout.setSpacing(12)

        title = QLabel("Strategies")
        title.setFont(QFont("Inter", 12, QFont.Medium))
        title.setStyleSheet("""
            color: white;
            background-color: #101010;
        """)
        layout.addWidget(title)

        self.strategy_list = QComboBox()
        self.strategy_list.setCursor(Qt.PointingHandCursor)
        self.strategy_list.setFont(QFont("Inter", 10))
        self.strategy_list.setStyleSheet("""
            QComboBox {
                background-color: #080808;
                border: 1px solid #363636;
                border-radius: 8px;
                padding: 8px;
                color: white;
            }
            QComboBox::drop-down {
                border: none;
                subcontrol-origin: padding;
                subcontrol-position: top right;
                width: 20px;
            }
            QComboBox::down-arrow {
                image: url(../../resources/images/down-arrow.svg);
                width: 16px;
                height: 16px;
                margin-right: 16px;
            }
            QComboBox QAbstractItemView {
                background-color: #080808;
                selection-background-color: #363636;
                color: white;
            }
        """)
        self.strategy_list.currentIndexChanged.connect(self.on_strategy_changed)
        layout.addWidget(self.strategy_list)

        self.log_card = LogCard()
        layout.addWidget(self.log_card, stretch=2)

        open_card = QFrame()
        open_card.setStyleSheet("""
            QFrame {
                background-color: #080808;
                border: 1px solid #363636;
                border-radius: 12px;
            }
        """)
        open_layout = QVBoxLayout(open_card)
        open_layout.setContentsMargins(8, 8, 8, 8)
        self.open_pnl_value = self.row(open_layout, "Open $ Profit/Loss", "--", "#999999", "#00C278")
        self.open_trade_value = self.row(open_layout, "Open Trade", "0", "#999999", "white")
        layout.addWidget(open_card)

        risk_card = QFrame()
        risk_card.setStyleSheet("""
            QFrame {
                background-color: #080808;
                border: 1px solid #363636;
                border-radius: 12px;
            }
        """)
        risk_layout = QVBoxLayout(risk_card)
        risk_layout.setContentsMargins(8, 8, 8, 8)

        self.risk_reward_value = self.row(risk_layout, "Risk/Reward", "--", "#00C278", "white")
        self.avg_win_value = self.row(risk_layout, "Avg. Win", "--", "#00C278", "white")
        self.avg_loss_value = self.row(risk_layout, "Avg. Loss", "--", "#EB5757", "white")
        self.max_drawdown_value = self.row(risk_layout, "Max Drawdown", "--", "#EB5757", "white")

        layout.addWidget(risk_card)
        layout.addStretch()

    def row(self, parent_layout, label, value, label_color, value_color):
        row = QHBoxLayout()

        left = QLabel(label)
        left.setFont(QFont("Inter", 10, QFont.Normal))
        left.setStyleSheet(f"color: {label_color}; border: none;")

        right = QLabel(value)
        right.setFont(QFont("Inter", 10, QFont.Normal))
        right.setStyleSheet(f"color: {value_color}; border: none;")
        right.setAlignment(Qt.AlignmentFlag.AlignRight)

        row.addWidget(left)
        row.addWidget(right)
        parent_layout.addLayout(row)

        return right

    def on_strategy_changed(self):
        self.current_bot_id = self.strategy_list.currentData()

    def set_active_strategies(self, runners):
        current_bot_id = self.current_bot_id

        self.strategy_list.blockSignals(True)
        self.strategy_list.clear()

        for bot_id, runner in runners.items():
            stats = runner.get_stats()
            strategy_name = stats.get("strategy_name", "Unknown strategy")
            self.strategy_list.addItem(strategy_name, bot_id)

        self.strategy_list.blockSignals(False)

        if self.strategy_list.count() > 0:
            index = 0

            if current_bot_id is not None:
                for i in range(self.strategy_list.count()):
                    if self.strategy_list.itemData(i) == current_bot_id:
                        index = i
                        break

            self.strategy_list.setCurrentIndex(index)
            self.on_strategy_changed()
        else:
            self.current_bot_id = None
            self.clear_stats()
            self.clear_logs()

    def set_logs(self, logs):
        self.clear_logs()
        for log in logs:
            self.add_strategy_log(self.current_bot_id, log)

    def clear_stats(self):
        self.open_pnl_value.setText("--")
        self.open_trade_value.setText("0")
        self.risk_reward_value.setText("--")
        self.avg_win_value.setText("--")
        self.avg_loss_value.setText("--")
        self.max_drawdown_value.setText("--")

    def clear_logs(self):
        while self.log_card.log_layout.count():
            item = self.log_card.log_layout.takeAt(0)
            widget = item.widget()
            if widget:
                widget.deleteLater()

    def update_strategy_stats(self, stats):
        if stats is None:
            self.clear_stats()
            return

        # TODO: format values properly
        position = stats.get("position", 0.0)
        best_bid = stats.get("best_bid")
        best_ask = stats.get("best_ask")
        mid_price = stats.get("mid_price")

        self.open_trade_value.setText("1" if position != 0 else "0")

        if None not in (best_bid, best_ask):
            mid_price = (best_bid + best_ask) / 2.0
            pnl = position * mid_price
            self.open_pnl_value.setText(f"{pnl:.2f}")
        else:
            self.open_pnl_value.setText("--")

        self.risk_reward_value.setText("--")
        self.avg_win_value.setText("--")
        self.avg_loss_value.setText("--")
        self.max_drawdown_value.setText("--")

    def add_strategy_log(self, bot_id, message):
        if bot_id != self.current_bot_id:
            return

        timestamp = QTime.currentTime().toString("hh:mm")
        self.log_card.add_log(message, "white", timestamp)
