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
        self.setAttribute(Qt.WidgetAttribute.WA_StyledBackground, True)
        self.setStyleSheet("""
            SideBar {
                background-color: #101010;
                border-right: 1px solid #363636;
            }
        """)

        layout = QVBoxLayout()
        layout.setContentsMargins(10, 64, 10, 20)
        layout.setSpacing(30)
        layout.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignHCenter)

        self.dashboard_btn = self.create_button("dashboard")
        self.bot_btn = self.create_button("bot")
        self.strat_btn = self.create_button("strat")
        self.stats_btn = self.create_button("chart")

        self.buttons = [
            self.dashboard_btn,
            self.bot_btn,
            self.strat_btn,
            self.stats_btn
        ]

        for btn in self.buttons:
            layout.addWidget(btn)

        self.dashboard_btn.setChecked(True)
        self.update_icon(self.dashboard_btn, True)

        layout.addStretch()
        self.setLayout(layout)

    def create_button(self, image_name):
        btn = QPushButton()
        btn.setCheckable(True)
        btn.setAutoExclusive(True)
        btn.setCursor(Qt.PointingHandCursor)
        btn.setFixedSize(36, 36)

        btn.base_icon = image_name

        btn.setIcon(QIcon(f"../../resources/images/{image_name}.svg"))
        btn.setIconSize(QSize(20, 20))

        btn.setStyleSheet("""
            QPushButton {
                background-color: #101010;
                border: none;
                border-radius: 10px;
            }
            QPushButton:checked {
                background-color: #D9D9D9;
            }
        """)

        btn.toggled.connect(lambda checked, b=btn: self.update_icon(b, checked))

        return btn

    def update_icon(self, button, checked):
        if checked:
            icon_path = f"../../resources/images/{button.base_icon}_selected.svg"
        else:
            icon_path = f"../../resources/images/{button.base_icon}.svg"

        button.setIcon(QIcon(icon_path))