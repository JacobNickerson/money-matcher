import numpy as np
import time
import sqlite3
from pathlib import Path
from PyQt5.QtWidgets import ( 
    QWidget, QHBoxLayout, QVBoxLayout, QPushButton, QLabel,
    QSizePolicy, QFrame, QComboBox, QGridLayout, QTableView,
    QHeaderView, QDialog,  QLineEdit, QMessageBox
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter, QLinearGradient, QIcon, QBrush, QPen,
    QPainterPath, QRegion
)
from PyQt5.QtCore import (
    Qt, QSize, QRectF, pyqtSignal
)
import pyqtgraph as pg
import models.performance_model as performance_model

class DonutWidget(QWidget):
    def __init__(self, pct=0.0, parent=None):
        super().__init__(parent)
        self.pct = pct
        self.setFixedSize(130, 130)

    def set_pct(self, pct):
        self.pct = max(0.0, min(1.0, float(pct)))
        self.update()

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

        self.donut = DonutWidget(0.0)
        body.addWidget(self.donut)
        body.setAlignment(Qt.AlignCenter)

        rbox = QVBoxLayout()
        rbox.setSpacing(24)

        self.quantity_value = QLabel("0")
        self.fillrate_value = QLabel("0.0%")

        for name, sub, value_label in [
            ("Quantity", "Overall Amount", self.quantity_value),
            ("Fill Rate", "% of Successful", self.fillrate_value),
        ]:
            item = QWidget()
            il = QHBoxLayout(item)
            il.setContentsMargins(0, 0, 0, 0)

            col = QVBoxLayout()
            n = QLabel(name)
            n.setStyleSheet("color: #FFFFFF; background:transparent; border:none;")
            n.setFont(QFont("Inter", 12, QFont.Medium))
            s = QLabel(sub)
            s.setStyleSheet("color: #707070; background:transparent; border:none;")
            s.setFont(QFont("Inter", 10, QFont.Medium))
            col.addWidget(n)
            col.addWidget(s)
            il.addLayout(col)

            value_label.setFont(QFont("Inter", 10, QFont.Medium))
            value_label.setStyleSheet("""
                color: #00C277;
                background-color: #0B1811;
                border-radius: 4px;
                padding: 1px 6px;
                border: none;
            """)
            value_label.setAlignment(Qt.AlignCenter)
            value_label.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)

            il.addWidget(value_label, Qt.AlignRight)
            rbox.addWidget(item)

        body.addLayout(rbox)
        lay.addLayout(body)
        lay.addStretch()

    def update_data(self, fill_rate, orders_filled, orders_submitted):
        self.donut.set_pct(fill_rate)
        self.quantity_value.setText(f"{orders_filled}/{orders_submitted}")
        self.fillrate_value.setText(f"{fill_rate * 100:.1f}%")

class MetricCard(QFrame):
    def __init__(self, title):
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

        self.title_lbl = QLabel(title)
        self.title_lbl.setStyleSheet("color: #ffffff; background: transparent; border: none;")
        self.title_lbl.setFont(QFont("Inter", 12, QFont.Medium))
        label_lay.addWidget(self.title_lbl)

        row = QHBoxLayout()
        row.setSpacing(8)

        self.val_lbl = QLabel("$0.00")
        self.val_lbl.setStyleSheet("color: #ffffff; background: transparent; border: none;")
        self.val_lbl.setFont(QFont("Inter", 20, QFont.DemiBold))
        row.addWidget(self.val_lbl)

        self.delta_lbl = QLabel("0.00%")
        self.delta_lbl.setFont(QFont("Inter", 10))
        self.delta_lbl.setAlignment(Qt.AlignCenter)
        self.delta_lbl.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)
        row.addWidget(self.delta_lbl, alignment=Qt.AlignBottom)

        row.addStretch()
        label_lay.addLayout(row)
        lay.addWidget(label_container)

        self.pw = pg.PlotWidget()
        self.pw.setMinimumHeight(80)
        self.pw.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        self.pw.setBackground("#080808")
        self.pw.hideButtons()
        self.pw.setMenuEnabled(False)
        self.pw.setMouseEnabled(False, False)
        for ax in ("left", "bottom", "right", "top"):
            self.pw.hideAxis(ax)
        self.pw.getViewBox().setBorder(None)
        self.pw.getPlotItem().setContentsMargins(0, 0, 0, 0)
        self.pw.getViewBox().setDefaultPadding(0)
        self.pw.setClipToView(True)
        lay.addWidget(self.pw, stretch=1)

        self.update_data("$0.00", "0.00%", True, np.array([0, 0, 0, 0, 0]))

    def update_data(self, value, delta_text, positive, y_data):
        self.val_lbl.setText(value)
        self.delta_lbl.setText(delta_text)

        delta_color = "#00C277" if positive else "#EB5757"
        bg_color = "#0B1811" if positive else "#1E1010"
        self.delta_lbl.setStyleSheet(f"""
            color: {delta_color};
            background-color: {bg_color};
            border-radius: 4px;
            padding: 1px 6px;
        """)

        self.pw.clear()

        y_data = np.asarray(y_data, dtype=float)
        if len(y_data) == 0:
            y_data = np.array([0.0])

        x = np.arange(len(y_data))

        grad = QLinearGradient(0, 1, 0, 0)
        grad.setCoordinateMode(QLinearGradient.ObjectBoundingMode)
        c_top = QColor(delta_color)
        c_top.setAlpha(80)
        c_bot = QColor(delta_color)
        c_bot.setAlpha(0)
        grad.setColorAt(0.0, c_top)
        grad.setColorAt(1.0, c_bot)

        curve = pg.PlotDataItem(x, y_data)
        baseline = pg.PlotDataItem(x, np.zeros_like(y_data))
        fill = pg.FillBetweenItem(curve, baseline, brush=QBrush(grad))

        self.pw.addItem(fill)
        self.pw.addItem(curve)
        curve.setPen(pg.mkPen(color=delta_color, width=2))

        ymin = float(np.min(y_data))
        ymax = float(np.max(y_data))
        if ymin == ymax:
            ymin -= 1
            ymax += 1

        self.pw.setRange(xRange=(0, len(y_data) - 1), yRange=(ymin, ymax), padding=0)

    def resizeEvent(self, event):
        super().resizeEvent(event)
        radius = 16
        region = QRegion(self.rect(), QRegion.Rectangle)

        path = QPainterPath()
        path.addRoundedRect(QRectF(self.rect()), radius, radius)
        region = QRegion(path.toFillPolygon().toPolygon())
        
        self.setMask(region)

class AddBalanceModal(QDialog):
    balanceadded = pyqtSignal(float)

    def __init__(self, current_balance=0.0, parent=None):
        super().__init__(parent)
        self.current_balance = float(current_balance)

        self.setModal(True)
        self.setMinimumWidth(600)
        self.setStyleSheet("""
            QDialog {
                background-color: #101010;
                border: 1px solid #363636;
                border-radius: 16px;
            }
            QLabel {
                color: #FFFFFF;
                background: transparent;
            }
            QPushButton {
                border-radius: 8px;
                padding: 10px 14px;
                font-size: 12px;
                font-weight: 600;
            }
            QLineEdit {
                background-color: #080808;
                color: #FFFFFF;
                border: 1px solid #363636;
                border-radius: 8px;
                padding: 10px 12px;
                font-size: 12px;
            }
            QLineEdit:focus {
                border: 1px solid #FFFFFF;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(16)

        header = QHBoxLayout()
        header.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Add to Account Balance")
        title.setFont(QFont("Inter", 12, QFont.Medium))

        close_btn = QPushButton()
        close_btn.setIcon(QIcon("../../resources/images/x_symbol.svg"))
        close_btn.setFixedSize(24, 24)
        close_btn.setCursor(Qt.PointingHandCursor)
        close_btn.setStyleSheet("""
            QPushButton {
                background: #FFFFFF;
                border-radius: 6px;
                font-size: 14px;
                padding: 0px;
            }
            QPushButton:hover {
                background-color: #D9D9D9;
            }
            QPushButton:pressed {
                background-color: #D9D9D9;
            }
        """)
        close_btn.clicked.connect(self.reject)

        header.addWidget(title)
        header.addStretch()
        header.addWidget(close_btn)
        layout.addLayout(header)

        current_balance_label = QLabel(f"Current Balance: ${self.current_balance:,.2f}")
        current_balance_label.setFont(QFont("Inter", 11, QFont.Medium))
        current_balance_label.setStyleSheet("color: #707070;")
        layout.addWidget(current_balance_label)

        self.amount_widget, self.amount_input = self.input_field(
            "Amount to Add",
            "1000.00"
        )
        layout.addWidget(self.amount_widget)

        layout.addStretch()

        submit_btn = QPushButton("Add Balance")
        submit_btn.setMinimumHeight(32)
        submit_btn.setMaximumHeight(44)
        submit_btn.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        submit_btn.setFont(QFont("Inter", 12, QFont.DemiBold))
        submit_btn.setCursor(Qt.PointingHandCursor)
        submit_btn.setStyleSheet("""
            QPushButton {
                background-color: #FFFFFF;
                color: #080808;
                border-radius: 8px;
                border: none;
            }
            QPushButton:hover {
                background-color: #D9D9D9;
            }
            QPushButton:pressed {
                background-color: #D9D9D9;
            }
        """)
        submit_btn.clicked.connect(self.handle_add_balance)
        layout.addWidget(submit_btn)

    def input_field(self, label_text, placeholder):
        wrapper = QWidget()
        wrapper.setStyleSheet("background: transparent;")

        layout = QVBoxLayout(wrapper)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(8)

        label = QLabel(label_text)
        label.setFont(QFont("Inter", 10, QFont.Medium))
        label.setStyleSheet("color: #FFFFFF;")

        line_edit = QLineEdit()
        line_edit.setPlaceholderText(placeholder)
        line_edit.setFont(QFont("Inter", 11))
        line_edit.setMinimumHeight(42)

        layout.addWidget(label)
        layout.addWidget(line_edit)

        return wrapper, line_edit

    def handle_add_balance(self):
        raw_amount = self.amount_input.text().strip().replace(",", "")

        try:
            amount = float(raw_amount)
        except ValueError:
            QMessageBox.warning("Invalid Amount", "Please enter a valid number")
            return

        if amount <= 0:
            QMessageBox.warning("Invalid Amount", "Amount must be greater than 0")
            return

        self.balanceadded.emit(amount)
        self.accept()

class BalanceCard(QFrame):
    addBalanceRequested = pyqtSignal()
    def __init__(self):
        super().__init__()
        self.setStyleSheet("QFrame#balanceCard { background: #101010; border:1px solid #363636; border-radius: 16px; }")
        self.setObjectName("balanceCard")
        self.setFixedWidth(320)

        lay = QVBoxLayout(self)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(0)

        label_container = QWidget()
        label_lay = QVBoxLayout(label_container)
        label_lay.setContentsMargins(16, 14, 16, 8)
        label_lay.setSpacing(4)

        top = QHBoxLayout()
        tl = QLabel("Account Balance")
        tl.setStyleSheet("color: #FFFFFF; border:none;")
        tl.setFont(QFont("Inter", 12, QFont.Medium))
        top.addWidget(tl)
        top.addStretch()
        self.add_balance_btn = QPushButton("Add to Balance")
        self.add_balance_btn.setIcon(QIcon("../../resources/images/plus_normal.svg"))
        self.add_balance_btn.setIconSize(QSize(16, 16))
        self.add_balance_btn.setFont(QFont("Inter", 10, QFont.Medium))
        self.add_balance_btn.setCursor(Qt.PointingHandCursor)
        self.add_balance_btn.setStyleSheet("""
            QPushButton { 
                background: #080808; 
                color: #FFFFFF; 
                border: 1px solid #363636;
                border-radius: 6px; 
                padding: 5px 12px;
            }
            QPushButton:hover { background:#2a2b30; }
        """)
        self.add_balance_btn.clicked.connect(self.addBalanceRequested.emit)
        top.addWidget(self.add_balance_btn)
        label_lay.addLayout(top)

        mid = QHBoxLayout()
        mid.setSpacing(8)

        self.vl = QLabel("$0.00")
        self.vl.setStyleSheet("color:#ffffff; background:transparent; border:none;")
        self.vl.setFont(QFont("Inter", 20, QFont.DemiBold))

        self.dl = QLabel("0.00%")
        self.dl.setFont(QFont("Inter", 10))
        self.dl.setAlignment(Qt.AlignCenter)
        self.dl.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)

        mid.addWidget(self.vl)
        mid.addWidget(self.dl, alignment=Qt.AlignBottom)
        mid.addStretch()
        label_lay.addLayout(mid)

        lay.addWidget(label_container)

        self.pw = pg.PlotWidget()
        self.pw.setMinimumHeight(80)
        self.pw.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        self.pw.setBackground("#101010")
        self.pw.hideButtons()
        self.pw.setMenuEnabled(False)
        self.pw.setMouseEnabled(False, False)
        for ax in ("left", "bottom", "right", "top"):
            self.pw.hideAxis(ax)
        self.pw.getViewBox().setBorder(None)
        self.pw.getPlotItem().setContentsMargins(0, 0, 0, 0)
        self.pw.getViewBox().setDefaultPadding(0)
        self.pw.setClipToView(True)
        lay.addWidget(self.pw)

        self.update_data("$0.00", "0.00%", True, np.array([0, 0, 0, 0, 0]))

    def update_data(self, value, delta_text, positive, y_data):
        self.vl.setText(value)
        self.dl.setText(delta_text)

        delta_color = "#00C277" if positive else "#EB5757"
        bg_color = "#0B1811" if positive else "#1E1010"
        self.dl.setStyleSheet(f"""
            color: {delta_color};
            background-color: {bg_color};
            border-radius: 4px;
            padding: 1px 6px;
        """)

        self.pw.clear()

        y_data = np.asarray(y_data, dtype=float)
        if len(y_data) == 0:
            y_data = np.array([0.0])

        x = np.arange(len(y_data))

        grad = QLinearGradient(0, 1, 0, 0)
        grad.setCoordinateMode(QLinearGradient.ObjectBoundingMode)
        c_top = QColor("#00C277")
        c_top.setAlpha(80)
        c_bot = QColor("#00C277")
        c_bot.setAlpha(0)
        grad.setColorAt(0.0, c_top)
        grad.setColorAt(1.0, c_bot)

        curve = pg.PlotDataItem(x, y_data)
        baseline = pg.PlotDataItem(x, np.zeros_like(y_data))
        fill = pg.FillBetweenItem(curve, baseline, brush=QBrush(grad))

        self.pw.addItem(fill)
        self.pw.addItem(curve)
        curve.setPen(pg.mkPen(color="#00C277", width=2))

        ymin = float(np.min(y_data))
        ymax = float(np.max(y_data))
        if ymin == ymax:
            ymin -= 1
            ymax += 1

        self.pw.setRange(xRange=(0, len(y_data) - 1), yRange=(ymin, ymax), padding=0)
    
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

        self.value_labels = {}

        outer_lay = QHBoxLayout(self)
        outer_lay.setContentsMargins(20, 20, 20, 20)
        outer_lay.setSpacing(20)

        columns = [
            ["Closed Trades", "Time Run", "Sharpe Ratio", "Sortino Ratio", "Maximum Drawdown"],
            ["Avg Trade", "Avg Winning Trade", "Avg Losing Trade", "Largest Win", "Largest Loss"],
            ["Winning Trades", "Losing Trades", "Long Trades", "Short Trades", "Profit Factor"],
        ]

        for names in columns:
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

            for key in names:
                row_w = QWidget()
                rl = QHBoxLayout(row_w)
                rl.setContentsMargins(14, 9, 14, 9)

                kl = QLabel(key)
                kl.setStyleSheet("color: #FFFFFF; background:transparent; border:none;")
                kl.setFont(QFont("Inter", 10, QFont.Normal))

                vl = QLabel("--")
                vl.setStyleSheet("color:#ffffff; background:transparent; border:none;")
                vl.setFont(QFont("Inter", 10, QFont.Normal))

                self.value_labels[key] = vl

                rl.addWidget(kl)
                rl.addStretch()
                rl.addWidget(vl)
                vbox.addWidget(row_w, stretch=1)

            outer_lay.addWidget(col_frame, stretch=1)

    def update_data(self, summary):
        self.value_labels["Closed Trades"].setText(str(summary.get("closed_trades", 0)))
        self.value_labels["Time Run"].setText(self.format_duration(summary.get("time_run_seconds", 0.0)))
        self.value_labels["Sharpe Ratio"].setText(f'{summary.get("sharpe_ratio", 0.0):.3f}')
        self.value_labels["Sortino Ratio"].setText(f'{summary.get("sortino_ratio", 0.0):.3f}')
        self.value_labels["Maximum Drawdown"].setText(f'-{summary.get("max_drawdown_pct", 0.0):.2f}%')

        self.value_labels["Avg Trade"].setText(f'{summary.get("avg_trade", 0.0):.2f}')
        self.value_labels["Avg Winning Trade"].setText(f'{summary.get("avg_winning_trade", 0.0):.2f}')
        self.value_labels["Avg Losing Trade"].setText(f'{summary.get("avg_losing_trade", 0.0):.2f}')
        self.value_labels["Largest Win"].setText(f'{summary.get("largest_win", 0.0):.2f}')
        self.value_labels["Largest Loss"].setText(f'{summary.get("largest_loss", 0.0):.2f}')

        self.value_labels["Winning Trades"].setText(str(summary.get("winning_trades", 0)))
        self.value_labels["Losing Trades"].setText(str(summary.get("losing_trades", 0)))

        trades = summary.get("trades", [])
        long_count = sum(1 for t in trades if t.get("side") == "BUY")
        short_count = sum(1 for t in trades if t.get("side") == "SELL")

        self.value_labels["Long Trades"].setText(str(long_count))
        self.value_labels["Short Trades"].setText(str(short_count))
        self.value_labels["Profit Factor"].setText(f'{summary.get("profit_factor", 0.0):.3f}')

    def format_duration(self, seconds):
        seconds = int(seconds)
        d, rem = divmod(seconds, 86400)
        h, rem = divmod(rem, 3600)
        m, _ = divmod(rem, 60)
        if d > 0:
            return f"{d}d {h}h {m}m"
        if h > 0:
            return f"{h}h {m}m"
        return f"{m}m"

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
        self.strategy_list.setMinimumWidth(400)
        layout.addWidget(self.strategy_list)

        self.date_list = QComboBox()
        self.date_list.setFont(QFont("Inter", 10, QFont.Medium))
        self.date_list.setStyleSheet(combo_style)
        self.date_list.setCursor(Qt.PointingHandCursor)
        self.date_list.addItems(["1 Day", "1 Week", "1 Month", "1 Year"])
        self.date_list.setMinimumWidth(200)
        layout.addWidget(self.date_list)

        self.refreshStrategyDropdown()

    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[2]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"

    def getStrategies(self):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            SELECT id, name, symbol, file_path
            FROM strategies
            ORDER BY name
        """)

        rows = cursor.fetchall()
        conn.close()
        return rows

    def refreshStrategyDropdown(self):
        current_data = self.strategy_list.currentData()

        self.strategy_list.blockSignals(True)
        self.strategy_list.clear()

        self.strategy_list.addItem("All Strategies", None)

        strategies = self.getStrategies()

        for strategy_id, name, symbol, file_path in strategies:
            label = f"{name} ({symbol})" if symbol else name
            self.strategy_list.addItem(label, {
                "strategy_id": strategy_id,
                "strategy_name": name,
                "symbol": symbol,
                "file_path": file_path,
            })

        if current_data is not None:
            for i in range(self.strategy_list.count()):
                if self.strategy_list.itemData(i) == current_data:
                    self.strategy_list.setCurrentIndex(i)
                    break

        self.strategy_list.blockSignals(False)

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

        # self.load_test_data()

    def update_trades(self, trades):
        rows = []
        for t in trades:
            entry_time = t.get("entry_time", 0)
            exit_time = t.get("exit_time", 0)

            rows.append({
                "Date": time.strftime("%b %d, %Y %I:%M %p", time.localtime(exit_time or entry_time)),
                "Strategy": t.get("bot_name", ""),
                "Symbol": t.get("symbol", ""),
                "Type": t.get("side", ""),
                "Profit/Loss": f'{t.get("pnl", 0.0):+.2f}',
                "% Gain/Loss": f'{t.get("pnl_pct", 0.0):+.2f}%',
                "Entry Price": f'{t.get("entry_price", 0.0):.2f}',
                "Exit Price": f'{t.get("exit_price", 0.0):.2f}',
            })

        self.model = performance_model.PerformanceModel(rows)
        self.table.setModel(self.model)

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
    def __init__(self, performance_tracker):
        super().__init__()

        self.performance_tracker = performance_tracker
        self.setStyleSheet("background-color: #0c0c0c;")

        grid = QGridLayout(self)
        grid.setRowStretch(0, 0)
        grid.setRowStretch(1, 1)
        grid.setRowStretch(2, 1)
        grid.setRowStretch(3, 1)
        grid.setColumnStretch(0, 1)
        grid.setColumnStretch(1, 1)
        grid.setColumnStretch(2, 1)
        grid.setSpacing(24)
        grid.setContentsMargins(0, 24, 0, 0)

        self.header = Header()
        grid.addWidget(self.header, 0, 0, 1, 3)

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

        self.profit_card = MetricCard("Profit/Loss")
        self.volume_card = MetricCard("Volume")
        self.fill_card = OrderFillCard()

        for card in (self.profit_card, self.volume_card, self.fill_card):
            top_layout.addWidget(card, stretch=1)

        grid.addWidget(top_frame, 1, 0, 1, 3)

        self.balance_card = BalanceCard()
        self.balance_card.addBalanceRequested.connect(self.open_add_balance_modal)
        grid.addWidget(self.balance_card, 2, 0)

        self.stats_card = StatsPanel()
        grid.addWidget(self.stats_card, 2, 1, 1, 2)

        self.order_panel = OrderHistory()
        grid.addWidget(self.order_panel, 3, 0, 1, 3)

        self.header.strategy_list.currentIndexChanged.connect(self.refresh_view)

        self.performance_tracker.performance_updated.connect(self.on_account_update)
        self.performance_tracker.bot_updated.connect(self.on_bot_update)
        self.performance_tracker.trade_opened.connect(self.on_trade_changed)
        self.performance_tracker.trade_closed.connect(self.on_trade_changed)

        self.refresh_view()

    def on_account_update(self, _summary):
        if self.selected_strategy_name() is None:
            self.refresh_view()

    def on_bot_update(self, _bot_id, _summary):
        selected_strategy = self.selected_strategy_name()
        if selected_strategy is not None:
            self.refresh_view()

    def on_trade_changed(self, _trade):
        self.refresh_view()

    def selected_strategy_meta(self):
        data = self.header.strategy_list.currentData()
        return data if isinstance(data, dict) else None

    def selected_strategy_name(self):
        meta = self.selected_strategy_meta()
        if not meta:
            return None
        return meta.get("strategy_name")
    
    def current_summary(self):
        strategy_name = self.selected_strategy_name()
        if strategy_name is None:
            return self.performance_tracker.get_account_summary()

        bot_summaries = self.performance_tracker.get_all_bot_summaries()
        matching = [
            s for s in bot_summaries
            if s.get("strategy_name") == strategy_name
        ]
        return self.aggregate_bot_summaries(matching, strategy_name)
    
    def refresh_view(self):
        summary = self.current_summary()
        self.apply_summary(summary)

    def apply_summary(self, summary):
        realized = float(summary.get("realized_pnl", 0.0))
        equity = float(summary.get("equity", 0.0))
        base_balance = float(summary.get("initial_balance", summary.get("allocated_balance", 0.0)))
        total_volume = float(summary.get("total_volume", 0.0))
        fill_rate = float(summary.get("fill_rate", 0.0))

        pnl_pct = float(summary.get("pnl_pct", 0.0))
        equity_pct = float(summary.get("equity_pct", 0.0))

        equity_history = summary.get("equity_history", [])
        balance_history = summary.get("balance_history", [])

        profit_series = self.series_from_history(equity_history, baseline=base_balance)
        balance_series = self.series_from_history(balance_history)
        volume_series = self.volume_series(total_volume, size=max(2, len(balance_series)))

        self.profit_card.update_data(
            value=f"${realized:,.2f}",
            delta_text=f"{pnl_pct:+.2f}%",
            positive=(realized >= 0),
            y_data=profit_series,
        )

        self.volume_card.update_data(
            value=f"${total_volume:,.2f}",
            delta_text=f"{fill_rate * 100:.2f}% fill",
            positive=True,
            y_data=volume_series,
        )

        self.fill_card.update_data(
            fill_rate=fill_rate,
            orders_filled=int(summary.get("orders_filled", 0)),
            orders_submitted=int(summary.get("orders_submitted", 0)),
        )

        self.balance_card.update_data(
            value=f"${equity:,.2f}",
            delta_text=f"{equity_pct:+.2f}%",
            positive=(equity >= base_balance),
            y_data=balance_series,
        )

        self.stats_card.update_data(summary)
        self.order_panel.update_trades(summary.get("trades", []))

    def series_from_history(self, history, baseline=None):
        if not history:
            if baseline is None:
                return np.array([0.0, 0.0], dtype=float)
            return np.array([baseline, baseline], dtype=float)

        series = np.array([float(value) for _, value in history], dtype=float)

        if baseline is not None:
            series = series - baseline

        if len(series) == 1:
            series = np.array([series[0], series[0]], dtype=float)

        return series

    def volume_series(self, total_volume, size=20):
        size = max(2, int(size))
        return np.linspace(0.0, float(total_volume), size)
    
    def aggregate_bot_summaries(self, summaries, strategy_name):
        if not summaries:
            return {
                "scope": "strategy",
                "strategy_name": strategy_name,
                "allocated_balance": 0.0,
                "cash_balance": 0.0,
                "equity": 0.0,
                "realized_pnl": 0.0,
                "unrealized_pnl": 0.0,
                "pnl_pct": 0.0,
                "equity_pct": 0.0,
                "orders_submitted": 0,
                "orders_filled": 0,
                "orders_cancelled": 0,
                "fill_rate": 0.0,
                "total_volume": 0.0,
                "closed_trades": 0,
                "winning_trades": 0,
                "losing_trades": 0,
                "avg_trade": 0.0,
                "avg_winning_trade": 0.0,
                "avg_losing_trade": 0.0,
                "largest_win": 0.0,
                "largest_loss": 0.0,
                "profit_factor": 0.0,
                "max_drawdown_pct": 0.0,
                "sharpe_ratio": 0.0,
                "sortino_ratio": 0.0,
                "time_run_seconds": 0.0,
                "balance_history": [],
                "equity_history": [],
                "trades": [],
            }

        allocated_balance = sum(float(s.get("allocated_balance", 0.0)) for s in summaries)
        cash_balance = sum(float(s.get("cash_balance", 0.0)) for s in summaries)
        equity = sum(float(s.get("equity", 0.0)) for s in summaries)
        realized_pnl = sum(float(s.get("realized_pnl", 0.0)) for s in summaries)
        unrealized_pnl = sum(float(s.get("unrealized_pnl", 0.0)) for s in summaries)

        orders_submitted = sum(int(s.get("orders_submitted", 0)) for s in summaries)
        orders_filled = sum(int(s.get("orders_filled", 0)) for s in summaries)
        orders_cancelled = sum(int(s.get("orders_cancelled", 0)) for s in summaries)
        total_volume = sum(float(s.get("total_volume", 0.0)) for s in summaries)

        all_trades = []
        for s in summaries:
            for t in s.get("trades", []):
                trade = dict(t)
                trade.setdefault("strategy_name", strategy_name)
                all_trades.append(trade)

        winners = [t for t in all_trades if float(t.get("pnl", 0.0)) > 0]
        losers = [t for t in all_trades if float(t.get("pnl", 0.0)) < 0]

        avg_trade = (
            sum(float(t.get("pnl", 0.0)) for t in all_trades) / len(all_trades)
            if all_trades else 0.0
        )
        avg_winning_trade = (
            sum(float(t.get("pnl", 0.0)) for t in winners) / len(winners)
            if winners else 0.0
        )
        avg_losing_trade = (
            sum(float(t.get("pnl", 0.0)) for t in losers) / len(losers)
            if losers else 0.0
        )
        largest_win = max((float(t.get("pnl", 0.0)) for t in winners), default=0.0)
        largest_loss = min((float(t.get("pnl", 0.0)) for t in losers), default=0.0)

        gross_profit = sum(float(t.get("pnl", 0.0)) for t in winners)
        gross_loss = abs(sum(float(t.get("pnl", 0.0)) for t in losers))
        if gross_loss == 0:
            profit_factor = gross_profit if gross_profit > 0 else 0.0
        else:
            profit_factor = gross_profit / gross_loss

        balance_history = self._merge_histories(
            [s.get("balance_history", []) for s in summaries]
        )
        equity_history = self._merge_histories(
            [s.get("equity_history", []) for s in summaries]
        )

        fill_rate = (orders_filled / orders_submitted) if orders_submitted else 0.0
        pnl_pct = (realized_pnl / allocated_balance * 100.0) if allocated_balance else 0.0
        equity_pct = ((equity - allocated_balance) / allocated_balance * 100.0) if allocated_balance else 0.0
        time_run_seconds = max(float(s.get("time_run_seconds", 0.0)) for s in summaries)

        return {
            "scope": "strategy",
            "strategy_name": strategy_name,
            "allocated_balance": allocated_balance,
            "cash_balance": cash_balance,
            "equity": equity,
            "realized_pnl": realized_pnl,
            "unrealized_pnl": unrealized_pnl,
            "pnl_pct": pnl_pct,
            "equity_pct": equity_pct,
            "orders_submitted": orders_submitted,
            "orders_filled": orders_filled,
            "orders_cancelled": orders_cancelled,
            "fill_rate": fill_rate,
            "total_volume": total_volume,
            "closed_trades": len(all_trades),
            "winning_trades": len(winners),
            "losing_trades": len(losers),
            "avg_trade": avg_trade,
            "avg_winning_trade": avg_winning_trade,
            "avg_losing_trade": avg_losing_trade,
            "largest_win": largest_win,
            "largest_loss": largest_loss,
            "profit_factor": profit_factor,
            "max_drawdown_pct": max(float(s.get("max_drawdown_pct", 0.0)) for s in summaries),
            "sharpe_ratio": 0.0,
            "sortino_ratio": 0.0,
            "time_run_seconds": time_run_seconds,
            "balance_history": balance_history,
            "equity_history": equity_history,
            "trades": all_trades,
        }
    
    def merge_histories(self, histories):
        latest_values = []
        latest_ts = time.time()

        for history in histories:
            if history:
                latest_ts = max(latest_ts, float(history[-1][0]))
                latest_values.append(float(history[-1][1]))

        total = sum(latest_values)
        return [(latest_ts - 1, total), (latest_ts, total)]
    
    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[2]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"
    
    def open_add_balance_modal(self):
        summary = self.performance_tracker.get_account_summary()
        modal = AddBalanceModal(
            current_balance=float(summary.get("cash_balance", 0.0)),
            parent=self
        )
        modal.balanceadded.connect(self.handle_balance_added)
        modal.exec_()

    def handle_balance_added(self, amount):
        self.updateAccountBalance(amount)
        self.performance_tracker.add_account_balance(amount)

    def updateAccountBalance(self, amount):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            UPDATE account
            SET cash_balance = cash_balance + ?,
                updated_at = ?
            WHERE id = 1
        """, (amount, time.time()))

        conn.commit()
        conn.close()
