from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QLineEdit,
    QSizePolicy, QTableView, QStyledItemDelegate, QTabBar, QHeaderView,
    QFrame, QGridLayout, QComboBox, QScrollArea, QApplication, QStyle,
    QStyleOptionButton
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter, QIcon, QPen
)
from PyQt5.QtCore import (
    Qt, QRect, QSize, pyqtSignal
)
import models.bot_model as bot_model

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

        title = QLabel("Bots")
        title.setFont(QFont("Inter", 20, QFont.Bold))
        title.setStyleSheet("color: #FFFFFF; background: transparent;")
        layout.addWidget(title)

        spacer = QWidget()
        spacer.setSizePolicy(QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Expanding)
        spacer.setStyleSheet("background-color: transparent")
        layout.addWidget(spacer)

        self.new_btn = QPushButton("  Create New Bot")
        self.new_btn.setIcon(QIcon("../../resources/images/plus.svg"))

        self.new_btn.setCursor(Qt.PointingHandCursor)
        self.new_btn.setFixedHeight(40)
        self.new_btn.setSizePolicy(QSizePolicy.Policy.Preferred, QSizePolicy.Policy.Expanding)
        self.new_btn.setFont(QFont("Inter", 12, QFont.DemiBold))
        self.new_btn.setStyleSheet("""
            QPushButton {
                background-color: #FFFFFF;
                color: #080808;
                border: none;
                border-radius: 8px;
            }
        """)
        self.new_btn.setIconSize(QSize(16, 16))

        layout.addWidget(self.new_btn)

class StatusDelegate(QStyledItemDelegate):
    def paint(self, painter, option, index):
        value = index.data(Qt.DisplayRole)

        painter.save()

        rect = option.rect.adjusted(8, 10, -8, -10)

        if value == "Active":
            bg = QColor("#121F18")
            text_color = QColor("#00C278")
        else:
            bg = QColor("#080808")
            text_color = QColor("#999999")

        painter.setRenderHint(QPainter.Antialiasing)

        painter.setBrush(bg)
        painter.setPen(Qt.NoPen)
        painter.drawRoundedRect(rect, 8, 8)

        painter.setPen(text_color)
        font = QFont("Inter", 9, QFont.Medium)
        painter.setFont(font)

        painter.drawText(rect, Qt.AlignCenter, value)

        painter.restore()

class ActionsDelegate(QStyledItemDelegate):
    def paint(self, painter, option, index):
        painter.save()

        rect = option.rect
        icon_size = 16
        spacing = 10

        style = QApplication.style()

        pause_icon = style.standardIcon(QStyle.SP_MediaPause)
        settings_icon = style.standardIcon(QStyle.SP_FileDialogDetailedView)
        delete_icon = style.standardIcon(QStyle.SP_TrashIcon)

        icons = [pause_icon, settings_icon, delete_icon]

        total_width = len(icons) * icon_size + (len(icons) - 1) * spacing
        x = rect.center().x() - total_width // 2
        y = rect.center().y() - icon_size // 2

        for icon in icons:
            icon.paint(painter, QRect(x, y, icon_size, icon_size))
            x += icon_size + spacing

        painter.restore()

class CheckBoxHeader(QHeaderView):
    clicked = pyqtSignal(bool)

    def __init__(self, orientation, parent=None):
        super().__init__(orientation, parent)
        self._state = 0
        self.setSectionsClickable(True)

    def setCheckState(self, state):
        self._state = state
        self.updateSection(0)

    def paintSection(self, painter, rect, logicalIndex):
        super().paintSection(painter, rect, logicalIndex)

        if logicalIndex != 0:
            return

        painter.save()
        painter.setRenderHint(QPainter.Antialiasing)

        size = 16
        x = rect.center().x() - size // 2
        y = rect.center().y() - size // 2
        box_rect = QRect(x, y, size, size)

        painter.setBrush(QColor("#ffffff"))
        painter.setPen(Qt.NoPen)
        painter.drawRoundedRect(box_rect, 4, 4)

        if self._state == 2:
            pen = QPen(QColor("#080808"), 2)
            painter.setPen(pen)
            painter.drawLine(x + 4, y + 8, x + 7, y + 11)
            painter.drawLine(x + 7, y + 11, x + 12, y + 5)

        painter.restore()

    def mousePressEvent(self, event):
        index = self.logicalIndexAt(event.pos())
        if index == 0:
            checked = self._state != 2
            self._state = 2 if checked else 0
            self.clicked.emit(checked)
            self.updateSection(0)
        super().mousePressEvent(event)

class CheckBoxDelegate(QStyledItemDelegate):
    def paint(self, painter, option, index):
        checked = index.data(Qt.CheckStateRole) == Qt.Checked

        painter.save()
        painter.setRenderHint(QPainter.Antialiasing)

        size = 16
        x = option.rect.center().x() - size // 2
        y = option.rect.center().y() - size // 2
        box_rect = QRect(x, y, size, size)

        painter.setBrush(QColor("#ffffff"))
        painter.setPen(Qt.NoPen)
        painter.drawRoundedRect(box_rect, 4, 4)

        if checked:
            pen = QPen(QColor("#080808"), 2)
            painter.setPen(pen)
            painter.drawLine(x + 4, y + 8, x + 7, y + 11)
            painter.drawLine(x + 7, y + 11, x + 12, y + 5)

        painter.restore()

class BotList(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)

        self.setStyleSheet("""
            BotList {
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

        self.load_test_data()
        self.table.horizontalHeader().viewport().update()

        layout.addWidget(self.table)

    def load_test_data(self):
        rows = []
        for i in range(8):
            rows.append({
                "Bot Name": "Matchmaker",
                "Strategy": "Momentum",
                "Symbol": "SOL/USD",
                "Latency": "450ms",
                "Jitter": "3ms",
                "Status": "Active" if i % 2 else "Paused",
                "Actions": ""
            })

        model = bot_model.BotModel(rows)
        self.table.setModel(model)

        header = CheckBoxHeader(Qt.Horizontal, self.table)
        self.table.setHorizontalHeader(header)
        header.setSectionResizeMode(0, QHeaderView.Fixed)
        self.table.setColumnWidth(0, 40)

        for i in range(1, model.columnCount()):
            header.setSectionResizeMode(i, QHeaderView.Stretch)

        header.clicked.connect(model.set_all_checked)
        model.headerCheckStateChanged.connect(header.setCheckState)

        checkbox_delegate = CheckBoxDelegate(self.table)
        self.table.setItemDelegateForColumn(0, checkbox_delegate)

        status_delegate = StatusDelegate(self.table)
        self.table.setItemDelegateForColumn(6, status_delegate)

        actions_delegate = ActionsDelegate(self.table)
        self.table.setItemDelegateForColumn(7, actions_delegate)