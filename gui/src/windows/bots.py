import sqlite3
from pathlib import Path
from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QLineEdit,
    QSizePolicy, QTableView, QStyledItemDelegate, QTabBar, QHeaderView,
    QFrame, QGridLayout, QComboBox, QScrollArea, QApplication, QStyle,
    QStyleOptionButton, QDialog, QMessageBox
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter, QIcon, QPen, QCursor
)
from PyQt5.QtCore import (
    Qt, QRect, QSize, pyqtSignal, QEvent
)
import models.bot_model as bot_model

class CreateBotModal(QDialog):
    botcreated = pyqtSignal()
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

        header = QHBoxLayout()
        header.setContentsMargins(0, 0, 0, 0)

        title = QLabel("Create New Bot")
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

        self.name_widget, self.name_input = self.input_field("Bot Name", " ")
        self.strategy_widget, self.strategy_list = self.strategy_dropdown()
        self.order_size_widget, self.order_size_input = self.input_field("Order Size", "1.00")
        self.max_position_widget, self.max_position_input = self.input_field("Max. Position", "100")
        self.latency_widget, self.latency_input = self.input_field("Latency (ms)", "3")
        self.jitter_widget, self.jitter_input = self.input_field("Jitter (ms)", "3")

        layout.addWidget(self.name_widget)
        layout.addWidget(self.strategy_widget)
        layout.addWidget(self.order_size_widget)
        layout.addWidget(self.max_position_widget)
        layout.addWidget(self.latency_widget)
        layout.addWidget(self.jitter_widget)

        layout.addStretch()

        submit_btn = QPushButton("Save Bot")
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
        submit_btn.clicked.connect(self.handle_save)

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

        return container, field
    
    def strategy_dropdown(self):
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

        self.strategy_list = QComboBox()
        self.strategy_list.setCursor(Qt.PointingHandCursor)
        self.strategy_list.setFont(QFont("Inter", 10))
        self.strategy_list.setMinimumHeight(30)
        self.strategy_list.setMaximumHeight(36)

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

        layout.addWidget(label)
        layout.addWidget(self.strategy_list)

        self.refreshStrategyDropdown()

        return container, self.strategy_list
    
    def get_data(self):
        strategy_name = self.strategy_list.currentText()
        strategy_file_path = self.strategy_list.currentData()
        symbol = self.getSymbolFromStrategy(strategy_file_path)

        return {
            "bot_name": self.name_input.text().strip(),
            "strategy_name": strategy_name,
            "strategy_file_path": strategy_file_path,
            "symbol": symbol,
            "order_size": self.order_size_input.text().strip(),
            "max_position": self.max_position_input.text().strip(),
            "latency": self.latency_input.text().strip(),
            "jitter": self.jitter_input.text().strip(),
        }
    
    def refreshStrategyDropdown(self):
        self.strategy_list.blockSignals(True)
        self.strategy_list.clear()

        strategies = self.getStrategies()

        for _, name, symbol, file_path in strategies:
            self.strategy_list.addItem(f"{name} ({symbol})", file_path)

        self.strategy_list.blockSignals(False)

    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[2]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"

    def saveBotToDatabase(self, data):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            INSERT INTO bots (
                bot_name,
                strategy_name,
                symbol,
                strategy_file_path,
                order_size,
                max_position,
                latency,
                jitter,
                status
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            data["bot_name"],
            data["strategy_name"],
            data["symbol"],
            data["strategy_file_path"],
            float(data["order_size"]),
            float(data["max_position"]),
            int(data["latency"]),
            int(data["jitter"]),
            "Paused"
        ))

        conn.commit()
        conn.close()

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

    def getSymbolFromStrategy(self, strategy_file_path):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            SELECT symbol
            FROM strategies
            WHERE file_path = ?
        """, (strategy_file_path,))

        result = cursor.fetchone()
        conn.close()
        return result[0] if result else ""
    
    def handle_save(self):
        try:
            data = self.get_data()
            self.saveBotToDatabase(data)
            self.botcreated.emit()
            self.accept()
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Could not create bot:\n{e}")

class Header(QWidget):
    def __init__(self, bot_list):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setStyleSheet("""
            Header {
                background-color: #101010;
                border: 1px solid #363636;
                border-radius: 16px;
            }
        """)
        self.bot_list = bot_list

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
        dialog.botcreated.connect(self.bot_list.loadBots)
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
        self.row_states = {}
        self.pause_icon = QIcon("../../resources/images/pause.svg")
        self.play_icon = QIcon("../../resources/images/play.svg")
        self.settings_icon = QIcon("../../resources/images/settings.svg")
        self.copy_icon = QIcon("../../resources/images/copy.svg")
        self.delete_icon = QIcon("../../resources/images/delete_white.svg")
        self.hovered_row = -1
        self.hovered_icon = -1

    def paint(self, painter, option, index):
        painter.save()

        rect = option.rect
        icon_size = 20
        spacing = 12

        state = self.row_states.get(index.row(), "active")

        pause_or_play = self.pause_icon if state == "active" else self.play_icon
        icons = [pause_or_play, self.copy_icon, self.settings_icon, self.delete_icon]

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

    def editorEvent(self, event, model, option, index):
        rect = option.rect
        icon_size = 20
        spacing = 12

        icons = ["pause", "copy", "settings", "delete"]

        total_width = len(icons) * icon_size + (len(icons) - 1) * spacing
        x_start = rect.center().x() - total_width // 2
        y = rect.center().y() - icon_size // 2

        if event.type() == QEvent.MouseMove:
            hovered_icon = -1
            x = x_start

            for i in range(len(icons)):
                icon_rect = QRect(x, y, icon_size, icon_size)

                if icon_rect.contains(event.pos()):
                    hovered_icon = i
                    break

                x += icon_size + spacing

            if self.hovered_row != index.row() or self.hovered_icon != hovered_icon:
                self.hovered_row = index.row()
                self.hovered_icon = hovered_icon
                self.parent().viewport().update()

            if hovered_icon != -1:
                self.parent().setCursor(QCursor(Qt.PointingHandCursor))
            else:
                self.parent().unsetCursor()

            return True

        if event.type() == QEvent.MouseButtonRelease:
            x = x_start

            for i, action in enumerate(icons):
                icon_rect = QRect(x, y, icon_size, icon_size)

                if icon_rect.contains(event.pos()):
                    self.handleAction(action, index)
                    return True

                x += icon_size + spacing

        return False
    
    def leaveEvent(self, event):
        self.unsetCursor()
        super().leaveEvent(event)
    
    def handleAction(self, action, index):
        row = index.row()

        if action == "pause":
            current = self.row_states.get(row, "active")

            if current == "active":
                self.row_states[row] = "paused"
                print("Paused bot", row)
            else:
                self.row_states[row] = "active"
                print("Started bot", row)

        elif action == "delete":
            model = index.model()
            row = index.row()

            bot_id = model.rows[row]["id"]
            bot_name = model.rows[row]["Bot Name"]

            reply = QMessageBox.question(
                None,
                "Confirm Delete",
                f"Are you sure you want to delete:\n\n{bot_name}?",
                QMessageBox.Discard | QMessageBox.Cancel,
                QMessageBox.Cancel
            )

            if reply != QMessageBox.Discard:
                return

            try:
                self.deleteBotFromDatabase(bot_id)
                self.parent().parent().loadBots()

            except Exception as e:
                QMessageBox.critical(
                    self.parent(),
                    "Error",
                    f"Could not delete bot:\n{e}"
                )

        elif action == "copy":
            print("Copy bot", row)

        elif action == "settings":
            print("Open settings", row)

        self.parent().viewport().update()

    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[2]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"

    def deleteBotFromDatabase(self, bot_id):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("DELETE FROM bots WHERE id = ?", (bot_id,))

        conn.commit()
        conn.close()

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
        self.initBotDatabase()

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

        self.header = CheckBoxHeader(Qt.Horizontal, self.table)
        self.table.setHorizontalHeader(self.header)

        self.checkbox_delegate = CheckBoxDelegate(self.table)
        self.status_delegate = StatusDelegate(self.table)
        self.actions_delegate = ActionsDelegate(self.table)

        self.table.setItemDelegateForColumn(0, self.checkbox_delegate)
        self.table.setItemDelegateForColumn(6, self.status_delegate)
        self.table.setItemDelegateForColumn(7, self.actions_delegate)

        #self.load_test_data()
        self.loadBots()
        self.table.horizontalHeader().viewport().update()

        layout.addWidget(self.table)

    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[2]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"
    
    def initBotDatabase(self):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            CREATE TABLE IF NOT EXISTS bots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bot_name TEXT NOT NULL,
                strategy_name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                strategy_file_path TEXT NOT NULL,
                order_size REAL NOT NULL,
                max_position REAL NOT NULL,
                latency INTEGER NOT NULL,
                jitter INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'Paused'
            )
        """)

        conn.commit()
        conn.close()
    
    def loadBots(self):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            SELECT id, bot_name, strategy_name, symbol, latency, jitter, status
            FROM bots
            ORDER BY id DESC
        """)

        db_rows = cursor.fetchall()
        conn.close()

        rows = []
        for bot_id, bot_name, strategy_name, symbol, latency, jitter, status in db_rows:
            rows.append({
                "id": bot_id,
                "Bot Name": bot_name,
                "Strategy": strategy_name,
                "Symbol": symbol,
                "Latency": f"{latency}ms",
                "Jitter": f"{jitter}ms",
                "Status": status,
                "Actions": ""
            })

        model = bot_model.BotModel(rows)
        self.table.setModel(model)

        self.header.clicked.connect(model.set_all_checked)
        model.headerCheckStateChanged.connect(self.header.setCheckState)

        self.header.setSectionResizeMode(0, QHeaderView.Fixed)
        self.table.setColumnWidth(0, 40)

        for i in range(1, model.columnCount()):
            self.header.setSectionResizeMode(i, QHeaderView.Stretch)

        self.table.horizontalHeader().show()
        self.table.update()

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