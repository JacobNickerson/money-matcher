from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QPushButton
)
from PyQt5.QtGui import (
    QIcon
)
from PyQt5.QtCore import (
    Qt, QSize
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