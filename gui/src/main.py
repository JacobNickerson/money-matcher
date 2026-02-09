import sys
import os
import re
from time import sleep
import pandas as pd
from lightweight_charts.widgets import QtChart
from PyQt5.QtWidgets import ( 
    QApplication, QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QSizePolicy, QMainWindow
)
from PyQt5.QtGui import (
    QFont, QIcon, QPixmap
)
from PyQt5.QtCore import Qt, QSize

class SideBar(QWidget):
    def __init__(self):
        super().__init__()
        self.setFixedWidth(112)
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setStyleSheet("""
            SideBar {
                background-color: #101010;
                border-color: #363636;
                border-width: 1px;
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
        self.setFixedHeight(500)
        self.setFixedWidth(600)
        self.setStyleSheet("""
            MarketEvents {
                background-color: #101010;
                border-color: #363636;
                border-width: 1px;
                border-style: solid;
                border-radius: 8px;
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

        # MarketEvents
        self.market_events = MarketEvents()
        content_layout.addWidget(self.market_events)
        content_layout.addStretch()

        main_layout.addWidget(self.sidebar)
        main_layout.addWidget(content_widget)

        self.setLayout(main_layout)


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = EngineWindow()
    window.show()
    sys.exit(app.exec_())