from PyQt5.QtGui import (
    QFont, QColor
)
from PyQt5.QtCore import (
    Qt, QAbstractTableModel, QModelIndex, pyqtSignal
)

class BotModel(QAbstractTableModel):
    headers = [
        "", "Bot Name", "Strategy", "Symbol", "Latency", "Jitter", "Status", "Actions"
    ]
    headerCheckStateChanged = pyqtSignal(int)  

    def __init__(self, rows):
        super().__init__()
        self.rows = rows
        self.checked = [False] * len(rows)

    def rowCount(self, parent=QModelIndex()):
        return len(self.rows)

    def columnCount(self, parent=QModelIndex()):
        return len(self.headers)

    def data(self, index, role):
        if not index.isValid():
            return None

        row = self.rows[index.row()]
        col = self.headers[index.column()]

        if index.column() == 0:
            if role == Qt.CheckStateRole:
                return Qt.Checked if self.checked[index.row()] else Qt.Unchecked
            return None

        if role == Qt.DisplayRole:
            if col == "Actions":
                return ""
            return row.get(col, "")
        
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font

        if role == Qt.TextAlignmentRole:
            return Qt.AlignLeft | Qt.AlignVCenter

        return None

    def headerData(self, section, orientation, role):
        if orientation == Qt.Horizontal:
            if section == 0 and role == Qt.CheckStateRole:
                if all(self.checked):
                    return Qt.Checked
                else:
                    return Qt.Unchecked
            if role == Qt.DisplayRole:
                return self.headers[section]

        if role == Qt.TextAlignmentRole:
            if self.headers[section] == "Status" or self.headers[section] == "Actions":
                return Qt.AlignCenter | Qt.AlignVCenter
            return Qt.AlignLeft | Qt.AlignVCenter
        
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font
        
        return None
    
    def flags(self, index):
        if not index.isValid():
            return Qt.NoItemFlags

        if index.column() == 0:
            return Qt.ItemIsEnabled | Qt.ItemIsUserCheckable

        return Qt.ItemIsEnabled
    
    def setData(self, index, value, role):
        if index.column() == 0 and role == Qt.CheckStateRole:
            self.checked[index.row()] = value == Qt.Checked
            self.dataChanged.emit(index, index)
            self.update_header_checkbox()
            return True
        return False
    
    def set_all_checked(self, checked):
        for i in range(len(self.checked)):
            self.checked[i] = checked

        top_left = self.index(0, 0)
        bottom_right = self.index(len(self.checked) - 1, 0)

        self.dataChanged.emit(top_left, bottom_right)
        self.update_header_checkbox()

    def update_header_checkbox(self):
        if all(self.checked):
            state = 2
        elif any(self.checked):
            state = 1
        else:
            state = 0

        self.headerCheckStateChanged.emit(state)