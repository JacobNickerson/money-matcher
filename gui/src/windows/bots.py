from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QLineEdit,
    QSizePolicy, QTableView, QStyledItemDelegate, QTabBar, QHeaderView,
    QFrame, QGridLayout, QComboBox, QScrollArea, QApplication, QStyle,
    QStyleOptionButton, QDialog
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter, QIcon, QPen, QCursor
)
from PyQt5.QtCore import (
    Qt, QRect, QSize, pyqtSignal, QEvent
)
import models.bot_model as bot_model

class CreateBotModal(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
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
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(16)

        title = QLabel("Create New Bot")
        title.setFont(QFont("Inter", 12, QFont.Medium))
        layout.addWidget(title)

        layout.addWidget(self.input_field("Bot Name", " "))
        layout.addWidget(self.strategy_list())
        layout.addWidget(self.input_field("First Order", "1.00"))
        layout.addWidget(self.input_field("Take Profit", "0.50"))
        layout.addWidget(self.input_field("Max. Extra Order", "5"))
        layout.addWidget(self.input_field("Percentage of Deposit", "1.00"))
        layout.addWidget(self.input_field("Latency", "3"))

        layout.addStretch()

        submit_btn = QPushButton("Create New Bot")
        submit_btn.setMinimumHeight(32)
        submit_btn.setMaximumHeight(44)
        submit_btn.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
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
        submit_btn.clicked.connect(self.accept)

        layout.addWidget(submit_btn)

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
                background-color: #080808;
                border: 1px solid #363636;
                border-radius: 6px;
                padding-right: 10px;
                color: white;
            }
        """)
        if (placeholder == " "):
            field.setAlignment(Qt.AlignLeft | Qt.AlignVCenter)
        else:
            field.setAlignment(Qt.AlignRight | Qt.AlignVCenter)

        field_layout = QHBoxLayout()
        field_layout.setContentsMargins(0, 0, 0, 0)
        field_layout.addWidget(field)

        layout.addWidget(label)
        layout.addLayout(field_layout)

        return container
    
    def strategy_list(self):
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
        
        label = QLabel("Strategy")
        label.setFont(QFont("Inter", 9))
        label.setStyleSheet("color: #999999;")

        list = QComboBox()
        list.setCursor(Qt.PointingHandCursor)
        list.setFont(QFont("Inter", 10))
        list.setMinimumHeight(30)
        list.setMaximumHeight(36)
        list.setStyleSheet("""
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
        list.addItems(["Momentum", "Arbitrage", "Scalping"])

        layout.addWidget(label)
        layout.addWidget(list)

        return container

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

        layout.addStretch()

        self.new_btn = QPushButton(" Create New Bot")
        self.new_btn.setIcon(QIcon("../../resources/images/plus.svg"))

        self.new_btn.setCursor(Qt.PointingHandCursor)
        self.new_btn.setFixedHeight(40)
        self.new_btn.setFixedWidth(190)
        self.new_btn.setIcon(QIcon("../../resources/images/plus_white.svg"))
        self.new_btn.setSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.Fixed)
        self.new_btn.setFont(QFont("Inter", 12, QFont.DemiBold))
        self.new_btn.setStyleSheet("""
            QPushButton {
                background-color: #FFFFFF;
                color: #080808;
                border: none;
                border-radius: 8px;
                padding: 0px 12px;
            }
        """)
        self.new_btn.setIconSize(QSize(20, 20))
        self.new_btn.clicked.connect(self.open_bot_modal)
        layout.addWidget(self.new_btn)

    def open_bot_modal(self):
        dialog = CreateBotModal(self)
        dialog.setWindowFlags(Qt.FramelessWindowHint | Qt.Dialog)
        dialog.exec()

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
    def __init__(self, parent=None):
        super().__init__(parent)

        self.pause_icon = QIcon("../../resources/images/pause.svg")
        self.settings_icon = QIcon("../../resources/images/settings.svg")
        self.copy_icon = QIcon("../../resources/images/copy.svg")
        self.delete_icon = QIcon("../../resources/images/delete_white.svg")

    def paint(self, painter, option, index):
        painter.save()

        rect = option.rect
        icon_size = 20
        spacing = 12

        icons = [self.pause_icon, self.copy_icon, self.settings_icon, self.delete_icon]

        total_width = (
            len(icons) * icon_size
            + (len(icons) - 1) * spacing
        )

        x = rect.center().x() - total_width // 2
        y = rect.center().y() - icon_size // 2

        pen = QPen(QColor("#363636"))
        pen.setWidth(1)
        painter.setPen(pen)

        for i, icon in enumerate(icons):
            icon.paint(painter, QRect(x, y, icon_size, icon_size))

            if i < len(icons) - 1:
                divider_x = x + icon_size + spacing // 2
                painter.drawLine(
                    divider_x,
                    y - 2,
                    divider_x,
                    y + icon_size + 2
                )

            x += icon_size + spacing

        painter.restore()

class CheckBoxHeader(QHeaderView):
    clicked = pyqtSignal(bool)

    def __init__(self, orientation, parent=None):
        super().__init__(orientation, parent)
        self._state = 0
        self.size = 16
        
        self.setSectionsClickable(True)
        self.setMouseTracking(True)
        self.viewport().setMouseTracking(True)

        self.unchecked_icon = QIcon("../../resources/images/checkbox.svg")
        self.checked_icon = QIcon("../../resources/images/checkbox_checked.svg")

    def setCheckState(self, state: int):
        self._state = state
        self.viewport().update()

    def sectionRect(self, logicalIndex: int) -> QRect:
        x = self.sectionPosition(logicalIndex)
        w = self.sectionSize(logicalIndex)
        return QRect(x, 0, w, self.height())

    def iconRect(self, logicalIndex: int) -> QRect:
        r = self.sectionRect(logicalIndex)
        return QRect(
            r.center().x() - self.size // 2,
            r.center().y() - self.size // 2,
            self.size,
            self.size
        )

    def hitRect(self, logicalIndex: int) -> QRect:
        r = self.iconRect(logicalIndex)
        shrink = 4
        return r.adjusted(shrink, shrink, -shrink, -shrink)

    def paintSection(self, painter, rect, logicalIndex):
        if logicalIndex != 0:
            super().paintSection(painter, rect, logicalIndex)
            return

        painter.save()
        super().paintSection(painter, rect, logicalIndex)
        painter.restore()

        painter.save()
        icon = self.checked_icon if self._state == 2 else self.unchecked_icon

        icon_rect = self.iconRect(0)
        painter.drawPixmap(icon_rect, icon.pixmap(self.size, self.size))
        painter.restore()

    def mouseMoveEvent(self, event):
        if self.logicalIndexAt(event.pos()) == 0 and self.hitRect(0).contains(event.pos()):
            self.viewport().setCursor(Qt.PointingHandCursor)
        else:
            self.viewport().unsetCursor()

        super().mouseMoveEvent(event)

    def leaveEvent(self, event):
        self.viewport().unsetCursor()
        super().leaveEvent(event)

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.LeftButton:
            if self.logicalIndexAt(event.pos()) == 0 and self.hitRect(0).contains(event.pos()):
                checked = self._state != 2
                self._state = 2 if checked else 0
                self.clicked.emit(checked)
                self.viewport().update()
                return

        super().mouseReleaseEvent(event)


class CheckBoxDelegate(QStyledItemDelegate):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.unchecked_icon = QIcon("../../resources/images/checkbox.svg")
        self.checked_icon = QIcon("../../resources/images/checkbox_checked.svg")
        self.size = 16

    def iconRect(self, option):
        x = option.rect.center().x() - self.size // 2
        y = option.rect.center().y() - self.size // 2
        return QRect(x, y, self.size, self.size)

    def paint(self, painter, option, index):
        checked = index.data(Qt.CheckStateRole) == Qt.Checked
        icon_rect = self.iconRect(option)
        icon = self.checked_icon if checked else self.unchecked_icon

        painter.save()
        icon.paint(painter, icon_rect, Qt.AlignCenter)
        painter.restore()

    def editorEvent(self, event, model, option, index):
        view = option.widget

        if event.type() == QEvent.MouseMove:
            if self.iconRect(option).contains(event.pos()):
                view.setCursor(Qt.PointingHandCursor)
            else:
                view.unsetCursor()

        if event.type() == QEvent.MouseButtonRelease and event.button() == Qt.LeftButton:
            if self.iconRect(option).contains(event.pos()):
                current = index.data(Qt.CheckStateRole)
                new_state = Qt.Unchecked if current == Qt.Checked else Qt.Checked
                return model.setData(index, new_state, Qt.CheckStateRole)

        return False

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
        self.table.setMouseTracking(True)
        self.table.horizontalHeader().setMouseTracking(True)

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