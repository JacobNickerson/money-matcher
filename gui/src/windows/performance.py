import numpy as np
from PyQt5.QtWidgets import ( 
    QWidget, QHBoxLayout, QVBoxLayout, QPushButton, QLabel,
    QSizePolicy, QFrame, QComboBox, QGridLayout, QTableView,
    QHeaderView
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter, QLinearGradient, QIcon, QBrush, QPen,
    QPainterPath, QRegion
)
from PyQt5.QtCore import (
    Qt, QSize, QRectF
)
import pyqtgraph as pg
import models.performance_model as performance_model

class DonutWidget(QWidget):
    def __init__(self, pct=0.50, parent=None):
        super().__init__(parent)
        self.pct = pct
        self.setFixedSize(130, 130)

    def paintEvent(self, _):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        cx, cy = self.width() / 2, self.height() / 2
        r = min(self.width(), self.height()) / 2 - 8
        rect = QRectF(cx - r, cy - r, 2 * r, 2 * r)

        p.setPen(QPen(QColor("#2a2b30"), 14, Qt.SolidLine, Qt.RoundCap))
        p.drawArc(rect, 0, 360 * 16)

        p.setPen(QPen(QColor("#00C277"), 14, Qt.SolidLine, Qt.RoundCap))
        p.drawArc(rect, 90 * 16, -int(self.pct * 360 * 16))

        p.setPen(QPen(QColor("#ffffff")))
        p.setFont(QFont("Segoe UI", 18, QFont.Bold))
        p.drawText(rect, Qt.AlignCenter, f"{int(self.pct * 100)}%")

class OrderFillCard(QFrame):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("""
            QFrame { 
                background: #080808; 
                border:1px solid #363636; 
                border-radius:12px; 
            }
        """)

        lay = QVBoxLayout(self)
        lay.setContentsMargins(16, 14, 16, 14)
        lay.setSpacing(12)

        title = QLabel("Order Fill Rate")
        title.setStyleSheet("color: #FFFFFF; background:transparent; border:none;")
        title.setFont(QFont("Inter", 12, QFont.Medium))
        lay.addWidget(title)
        lay.addStretch()

        body = QHBoxLayout()
        body.setSpacing(48)
        body.addWidget(DonutWidget(0.50))
        body.setAlignment(Qt.AlignCenter)

        rbox = QVBoxLayout()
        rbox.setSpacing(24)
        for name, sub, pct in [("Quantity", "Overall Amount", "+13.3%"), ("Fill Rate", "% of Successful", "+17.6%")]:
            item = QWidget()
            item.setStyleSheet("background:transparent;")
            il = QHBoxLayout(item)
            il.setContentsMargins(0, 0, 0, 0)
            il.setSpacing(24)
            col = QVBoxLayout()
            col.setSpacing(2)
            n = QLabel(name) 
            n.setStyleSheet("color: #FFFFFF; background:transparent; border:none;")
            n.setFont(QFont("Inter", 12, QFont.Medium))
            s = QLabel(sub) 
            s.setStyleSheet("color: #707070; background:transparent; border:none;")
            s.setFont(QFont("Inter", 10, QFont.Medium))
            col.addWidget(n)
            col.addWidget(s)
            il.addLayout(col)
            p = QLabel(pct) 
            p.setFont(QFont("Inter", 10, QFont.Medium))
            delta_color = "#00C277" if pct[0] == "+" else "#EB5757"
            bg_color = "#0B1811" if pct[0] == "+" else "#1E1010"
            p.setStyleSheet(f"""
                color: {delta_color};
                background-color: {bg_color};
                border-radius: 4px;
                padding: 1px 6px;
                border: none;
            """)
            p.setAlignment(Qt.AlignCenter)
            p.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)
            il.addWidget(p, Qt.AlignRight)
            rbox.addWidget(item)
        body.addLayout(rbox)
        container = QWidget()
        container.setLayout(body)
        container.setSizePolicy(QSizePolicy.Maximum, QSizePolicy.Preferred)
        container.setStyleSheet("background: transparent")
        lay.addWidget(container, alignment=Qt.AlignHCenter)
        lay.addStretch()

class MetricCard(QFrame):
    def __init__(self, title, value, delta, positive, y_data):
        super().__init__()
        self.setStyleSheet("""
            MetricCard { 
                background: #080808; 
                border: 1px solid #363636; 
                border-radius: 16px; 
            }
        """)

        lay = QVBoxLayout(self)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(0)

        label_container = QWidget()
        label_container.setStyleSheet("background: transparent;")
        label_lay = QVBoxLayout(label_container)
        label_lay.setContentsMargins(16, 14, 16, 8)
        label_lay.setSpacing(4)

        title_lbl = QLabel(title)
        title_lbl.setStyleSheet("color: #ffffff; background: transparent; border: none;")
        title_lbl.setFont(QFont("Inter", 12, QFont.Medium))
        label_lay.addWidget(title_lbl)

        row = QHBoxLayout()
        row.setSpacing(8)
        val_lbl = QLabel(value)
        val_lbl.setStyleSheet("color: #ffffff; background: transparent; border: none;")
        val_lbl.setFont(QFont("Inter", 20, QFont.DemiBold))
        row.addWidget(val_lbl)

        delta_color = "#00C277" if positive else "#EB5757"
        bg_color = "#0B1811" if positive else "#1E1010"
        delta_lbl = QLabel(delta)
        delta_lbl.setFont(QFont("Inter", 10))
        delta_lbl.setStyleSheet(f"""
            color: {delta_color};
            background-color: {bg_color};
            border-radius: 4px;
            padding: 1px 6px;
        """)

        delta_lbl.setAlignment(Qt.AlignCenter)
        delta_lbl.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)

        row.addWidget(delta_lbl, alignment=Qt.AlignBottom)
        row.addStretch()
        label_lay.addLayout(row)

        lay.addWidget(label_container)

        pw = pg.PlotWidget()
        pw.setMinimumHeight(80)
        pw.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        pw.setBackground("#080808")
        pw.hideButtons()
        pw.setMenuEnabled(False)
        pw.setMouseEnabled(False, False)
        for ax in ("left", "bottom", "right", "top"):
            pw.hideAxis(ax)
        pw.getViewBox().setBorder(None)
        pw.getPlotItem().setContentsMargins(0, 0, 0, 0)
        pw.getViewBox().setDefaultPadding(0)
        pw.setClipToView(True)

        x = np.arange(len(y_data))
        grad = QLinearGradient(0, 1, 0, 0)
        grad.setCoordinateMode(QLinearGradient.ObjectBoundingMode)
        c_top = QColor("#00C277" if positive else "#EB5757"); c_top.setAlpha(80)
        c_bot = QColor("#00C277" if positive else "#EB5757"); c_bot.setAlpha(0)
        grad.setColorAt(0.0, c_top)
        grad.setColorAt(1.0, c_bot)
        fill = pg.FillBetweenItem(
            pg.PlotDataItem(x, y_data),
            pg.PlotDataItem(x, np.zeros_like(y_data)),
            brush=QBrush(grad)
        )
        pw.addItem(fill)
        pw.plot(x, y_data, pen=pg.mkPen(color="#00C277" if positive else "#EB5757", width=2))
        pw.setRange(xRange=(0, len(y_data)-1), yRange=(y_data.min()-2, y_data.max()+2), padding=0)
        lay.addWidget(pw, stretch=1)

    def resizeEvent(self, event):
        super().resizeEvent(event)
        radius = 16
        region = QRegion(self.rect(), QRegion.Rectangle)

        path = QPainterPath()
        path.addRoundedRect(QRectF(self.rect()), radius, radius)
        region = QRegion(path.toFillPolygon().toPolygon())
        
        self.setMask(region)

class BalanceCard(QFrame):
    def __init__(self, value, delta, positive, y_data):
        super().__init__()
        self.setStyleSheet("QFrame#balanceCard { background: #101010; border:1px solid #363636; border-radius: 16px; }")
        self.setObjectName("balanceCard")
        self.setFixedWidth(320)

        lay = QVBoxLayout(self)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(0)

        label_container = QWidget()
        label_container.setStyleSheet("background: transparent;")
        label_lay = QVBoxLayout(label_container)
        label_lay.setContentsMargins(16, 14, 16, 8)
        label_lay.setSpacing(4)

        top = QHBoxLayout()
        tl = QLabel("Account Balance")
        tl.setStyleSheet("color: #FFFFFF; border:none;")
        tl.setFont(QFont("Inter", 12, QFont.Medium))
        top.addWidget(tl)
        top.addStretch()
        btn = QPushButton("Add to Balance")
        btn.setIcon(QIcon("../../resources/images/plus_normal.svg"))
        btn.setIconSize(QSize(16, 16))
        btn.setFont(QFont("Inter", 10, QFont.Medium))
        btn.setCursor(Qt.PointingHandCursor)
        btn.setStyleSheet("""
            QPushButton { 
                background: #080808; 
                color: #FFFFFF; 
                border: 1px solid #363636;
                border-radius: 6px; 
                padding: 5px 12px; }
            QPushButton:hover { background:#2a2b30; }
        """)
        top.addWidget(btn)
        label_lay.addLayout(top)

        delta_color = "#00C277" if positive else "#EB5757"
        bg_color = "#0B1811" if positive else "#1E1010"

        mid = QHBoxLayout()
        mid.setSpacing(8)
        vl = QLabel(value)
        vl.setStyleSheet("color:#ffffff; background:transparent; border:none;")
        vl.setFont(QFont("Inter", 20, QFont.DemiBold))
        dl = QLabel(delta)
        dl.setFont(QFont("Inter", 10))
        dl.setStyleSheet(f"""
            color: {delta_color};
            background-color: {bg_color};
            border-radius: 4px;
            padding: 1px 6px;
        """)
        dl.setAlignment(Qt.AlignCenter)
        dl.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)
        mid.addWidget(vl); 
        mid.addWidget(dl, alignment=Qt.AlignBottom); 
        mid.addStretch()
        label_lay.addLayout(mid)

        lay.addWidget(label_container)

        pw = pg.PlotWidget()
        pw.setMinimumHeight(80)
        pw.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        pw.setBackground("#101010")
        pw.hideButtons()
        pw.setMenuEnabled(False)
        pw.setMouseEnabled(False, False)
        for ax in ("left", "bottom", "right", "top"):
            pw.hideAxis(ax)
        pw.getViewBox().setBorder(None)
        pw.getPlotItem().setContentsMargins(0, 0, 0, 0)
        pw.getViewBox().setDefaultPadding(0)
        pw.setStyleSheet("""
            border: none;
        """)
        pw.setClipToView(True)

        x = np.arange(len(y_data))
        grad = QLinearGradient(0, 1, 0, 0)
        grad.setCoordinateMode(QLinearGradient.ObjectBoundingMode)
        c_top = QColor("#00C277"); c_top.setAlpha(80)
        c_bot = QColor("#00C277"); c_bot.setAlpha(0)
        grad.setColorAt(0.0, c_top); grad.setColorAt(1.0, c_bot)
        fill = pg.FillBetweenItem(
            pg.PlotDataItem(x, y_data),
            pg.PlotDataItem(x, np.zeros_like(y_data)),
            brush=QBrush(grad)
        )
        pw.addItem(fill)
        pw.plot(x, y_data, pen=pg.mkPen(color="#00C277", width=2))
        pw.setRange(xRange=(0, len(y_data)-1), yRange=(y_data.min()-2, y_data.max()+2), padding=0)
        lay.addWidget(pw)
    
    def resizeEvent(self, event):
        super().resizeEvent(event)
        radius = 16
        region = QRegion(self.rect(), QRegion.Rectangle)

        path = QPainterPath()
        path.addRoundedRect(QRectF(self.rect()), radius, radius)
        region = QRegion(path.toFillPolygon().toPolygon())
        self.setMask(region)

class StatsPanel(QFrame):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("""
            QFrame#statsOuter {
                background: #101010;
                border: 1px solid #363636;
                border-radius: 14px;
            }
        """)
        self.setObjectName("statsOuter")

        outer_lay = QHBoxLayout(self)
        outer_lay.setContentsMargins(20, 20, 20, 20)
        outer_lay.setSpacing(20)

        columns = [
            [("Closed Trades",    "18",          "#ffffff"),
             ("Time Run",         "3d 14h 12m",  "#ffffff"),
             ("Sharpe Ratio",     "1.376",        "#ffffff"),
             ("Sortino Ratio",    "1.664",        "#ffffff"),
             ("Maximum Drawdown", "-4.30%",       "#EB5757")],

            [("Avg Trade",         "+3.22%",  "#00C278"),
             ("Avg Winning Trade", "+9.41%",  "#00C278"),
             ("Avg Losing Trade",  "-3.28%",  "#EB5757"),
             ("Largest Win",       "+8.64%",  "#00C278"),
             ("Largest Loss",      "-8.39%",  "#EB5757")],

            [("Winning Trades",   "11",      "#ffffff"),
             ("Losing Trades",    "7",       "#ffffff"),
             ("Long Trades",      "13",      "#ffffff"),
             ("Short Trades",     "5",       "#ffffff"),
             ("Profit Factor",    "1.857",   "#ffffff")],
        ]

        for col_data in columns:
            col_frame = QFrame()
            col_frame.setStyleSheet("""
                QFrame {
                    background: #080808;
                    border: 1px solid #363636;
                    border-radius: 10px;
                }
            """)
            vbox = QVBoxLayout(col_frame)
            vbox.setContentsMargins(0, 8, 0, 8)
            vbox.setSpacing(0)

            for key, val, val_color in col_data:
                row_w = QWidget()
                row_w.setStyleSheet("background: transparent;")
                rl = QHBoxLayout(row_w)
                rl.setContentsMargins(14, 9, 14, 9)
                kl = QLabel(key)
                kl.setStyleSheet("color: #FFFFFF; background:transparent; border:none;")
                kl.setFont(QFont("Inter", 10, QFont.Normal))
                vl = QLabel(val)
                vl.setStyleSheet(f"color:{val_color};  background:transparent; border:none;")
                vl.setFont(QFont("Inter", 10, QFont.Normal))
                rl.addWidget(kl); rl.addStretch(); rl.addWidget(vl)

                vbox.addWidget(row_w, stretch=1)

            outer_lay.addWidget(col_frame, stretch=1)

class Header(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setStyleSheet("""
            Header {
                background-color: #101010;
                border: 1px solid #363636;
                border-radius: 16px;
            }
        """)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(12)

        title = QLabel("Performance Analytics")
        title.setFont(QFont("Inter", 20, QFont.Bold))
        title.setStyleSheet("color: #FFFFFF; background: transparent;")
        layout.addWidget(title)

        spacer = QWidget()
        spacer.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        spacer.setStyleSheet("background-color: transparent")
        layout.addWidget(spacer)

        combo_style = """
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
        """

        self.strategy_list = QComboBox()
        self.strategy_list.setFont(QFont("Inter", 10, QFont.Medium))
        self.strategy_list.setStyleSheet(combo_style)
        self.strategy_list.setCursor(Qt.PointingHandCursor)
        self.strategy_list.addItems(["Momentum", "Arbitrage", "Scalping"])
        self.strategy_list.setMinimumWidth(400)
        layout.addWidget(self.strategy_list)

        self.date_list = QComboBox()
        self.date_list.setFont(QFont("Inter", 10, QFont.Medium))
        self.date_list.setStyleSheet(combo_style)
        self.date_list.setCursor(Qt.PointingHandCursor)
        self.date_list.addItems(["1 Day", "1 Week", "1 Month", "1 Year"])
        self.date_list.setMinimumWidth(200)
        layout.addWidget(self.date_list)

class OrderHistory(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)

        self.setStyleSheet("""
            OrderHistory {
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

        self.load_test_data()

    def load_test_data(self):
        rows = []
        for i in range(8):
            rows.append({
                "Date": "Jan 26, 2025 5:30 PM",
                "Strategy": "Momentum",
                "Symbol": "SOL/USD",
                "Type": "Stop Limit" if i % 2 == 0 else "Limit",
                "Profit/Loss": "+911.78" if i % 3 else "-15.35",
                "% Gain/Loss": "17.8%" if i % 3 else "-0.3%",
                "Entry Price": "1489.36",
                "Exit Price": "1748.26",
            })

        model = performance_model.PerformanceModel(rows)
        self.table.setModel(model)

class Main(QWidget):
    def __init__(self):
        super().__init__()

        self.setStyleSheet("background-color: #0c0c0c;")

        np.random.seed(42)
        def smooth(n=120, trend=0.0):
            raw = np.cumsum(np.random.randn(n) * 0.5) + trend * np.linspace(0, 1, n)
            return raw - raw.min()

        grid = QGridLayout(self)
        grid.setRowStretch(0, 1)
        grid.setRowStretch(1, 1)
        grid.setRowStretch(2, 1)
        grid.setColumnStretch(0, 1)
        grid.setColumnStretch(1, 1)
        grid.setColumnStretch(2, 1)
        grid.setSpacing(24)
        grid.setContentsMargins(0, 24, 0, 0)

        top_frame = QFrame()
        top_frame.setStyleSheet("""
            QFrame#topFrame {
                background: #101010;
                border: 1px solid #363636;
                border-radius: 16px;
            }
        """)
        top_frame.setObjectName("topFrame")
        top_layout = QHBoxLayout(top_frame)
        top_layout.setContentsMargins(24, 24, 24, 24)
        top_layout.setSpacing(24)

        profit_card = MetricCard("Profit/Loss", "$1000.20", "+4.22 (4.23%)", True, smooth(120,  3))
        volume_card = MetricCard("Volume", "$41.99", "-1.54 (1.09%)", False, smooth(120, -1))
        fill_card   = OrderFillCard()

        for card in (profit_card, volume_card, fill_card):
            top_layout.addWidget(card, stretch=1)

        grid.addWidget(top_frame, 0, 0, 1, 3)

        balance_card = BalanceCard("$10,000.20", "+4.22 (4.23%)", True, smooth(200, 8))
        grid.addWidget(balance_card, 1, 0)

        stats_card = StatsPanel()
        grid.addWidget(stats_card, 1, 1, 1, 2)

        order_panel = OrderHistory()
        grid.addWidget(order_panel, 2, 0, 1, 3)
