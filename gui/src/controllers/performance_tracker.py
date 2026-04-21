import math
import time
from dataclasses import dataclass, field, replace
from typing import Dict, List, Optional, Tuple

from PyQt5.QtCore import QObject, pyqtSignal

@dataclass
class Trade:
    bot_id: int
    bot_name: str
    symbol: str
    order_id: int
    side: str
    entry_price: float
    entry_time: float
    entry_qty: float

    exit_price: float = 0.0
    exit_time: float = 0.0
    exit_qty: float = 0.0

    pnl: float = 0.0
    pnl_pct: float = 0.0

    @property
    def is_closed(self) -> bool:
        return self.exit_time > 0 and self.exit_qty > 0

    @property
    def remaining_qty(self) -> float:
        return max(0.0, self.entry_qty - self.exit_qty)

    @property
    def direction(self) -> int:
        return 1 if self.side.upper() == "BUY" else -1

    def unrealized_pnl(self, mark_price: float) -> float:
        if self.remaining_qty <= 0:
            return 0.0

        if self.side.upper() == "BUY":
            return (mark_price - self.entry_price) * self.remaining_qty
        return (self.entry_price - mark_price) * self.remaining_qty

    def close(
        self,
        exit_price: float,
        exit_time: float,
        exit_qty: Optional[float] = None,
    ) -> None:
        if exit_qty is None:
            exit_qty = self.remaining_qty

        exit_qty = float(exit_qty)

        if exit_qty <= 0:
            raise ValueError("exit_qty must be > 0")

        if exit_qty > self.remaining_qty + 1e-9:
            raise ValueError("exit_qty cannot be greater than remaining_qty")

        exit_price = float(exit_price)
        exit_time = float(exit_time)

        # Weighted average exit for multiple partial closes
        if self.exit_qty == 0:
            self.exit_price = exit_price
        else:
            total_closed_value = (self.exit_price * self.exit_qty) + (exit_price * exit_qty)
            self.exit_price = total_closed_value / (self.exit_qty + exit_qty)

        self.exit_qty += exit_qty
        self.exit_time = exit_time

        if self.side.upper() == "BUY":
            realized_piece = (exit_price - self.entry_price) * exit_qty
        else:
            realized_piece = (self.entry_price - exit_price) * exit_qty

        self.pnl += realized_piece

        total_entry_value_closed = self.entry_price * self.exit_qty
        if total_entry_value_closed > 0:
            self.pnl_pct = (self.pnl / total_entry_value_closed) * 100.0


@dataclass
class BotStats:
    bot_id: int
    bot_name: str
    symbol: str
    strategy_name: str = "Unknown"
    allocated_balance: float = 0.0

    orders_submitted: int = 0
    orders_filled: int = 0
    orders_cancelled: int = 0

    total_volume: float = 0.0
    total_realized_pnl: float = 0.0
    total_unrealized_pnl: float = 0.0

    current_position: float = 0.0
    mark_price: float = 0.0

    start_time: float = field(default_factory=time.time)

    open_trades: Dict[int, Trade] = field(default_factory=dict)
    closed_trades: List[Trade] = field(default_factory=list)
    all_trades: List[Trade] = field(default_factory=list)

    balance_history: List[Tuple[float, float]] = field(default_factory=list)
    equity_history: List[Tuple[float, float]] = field(default_factory=list)
    position_history: List[Tuple[float, float]] = field(default_factory=list)

    def __post_init__(self) -> None:
        initial_bot_balance = float(self.allocated_balance)
        self.balance_history.append((self.start_time, initial_bot_balance))
        self.equity_history.append((self.start_time, initial_bot_balance))
        self.position_history.append((self.start_time, 0.0))


class PerformanceTracker(QObject):
    performance_updated = pyqtSignal(dict)
    bot_updated = pyqtSignal(int, dict)
    trade_opened = pyqtSignal(dict)
    trade_closed = pyqtSignal(dict)

    def __init__(self, starting_balance: float):
        super().__init__()

        self.start_time = time.time()

        self.starting_balance = float(starting_balance)
        self.net_deposits = 0.0

        self.account_cash_balance = float(starting_balance)
        self.account_equity = float(starting_balance)

        self.account_peak_equity = float(starting_balance)
        self.account_peak_cash = float(starting_balance)

        self.account_max_drawdown_pct = 0.0
        self.account_current_drawdown_pct = 0.0

        self.account_realized_pnl = 0.0
        self.account_unrealized_pnl = 0.0

        self.account_balance_history: List[Tuple[float, float]] = [
            (self.start_time, self.account_cash_balance)
        ]
        self.account_equity_history: List[Tuple[float, float]] = [
            (self.start_time, self.account_equity)
        ]

        self.bots: Dict[int, BotStats] = {}
        self.order_to_bot: Dict[int, int] = {}
        self.order_history: List[dict] = []

    def register_bot(
        self,
        bot_id: int,
        bot_name: str,
        symbol: str,
        allocated_balance: float = 0.0,
        strategy_name: str = "Unknown",
    ) -> None:
        if bot_id in self.bots:
            return

        self.bots[bot_id] = BotStats(
            bot_id=bot_id,
            bot_name=bot_name,
            symbol=symbol,
            strategy_name=strategy_name,
            allocated_balance=float(allocated_balance),
        )

        self._emit_bot_update(bot_id)
        self._emit_global_update()

    def unregister_bot(self, bot_id: int) -> None:
        if bot_id not in self.bots:
            return

        bot = self.bots[bot_id]
        if bot.open_trades:
            raise ValueError("Cannot unregister bot with open trades")

        del self.bots[bot_id]

        stale_order_ids = [
            order_id
            for order_id, owner_bot_id in self.order_to_bot.items()
            if owner_bot_id == bot_id
        ]
        for order_id in stale_order_ids:
            del self.order_to_bot[order_id]

        self._emit_global_update()

    def record_order_submission(self, bot_id: int) -> None:
        bot = self._get_bot(bot_id)
        bot.orders_submitted += 1
        self._emit_bot_update(bot_id)
        self._emit_global_update()

    def record_order_fill(self, bot_id: int) -> None:
        bot = self._get_bot(bot_id)
        bot.orders_filled += 1
        self._emit_bot_update(bot_id)
        self._emit_global_update()

    def record_fill_volume(self, bot_id: int, price: float, qty: float) -> None:
        bot = self._get_bot(bot_id)
        bot.total_volume += abs(float(price) * float(qty))
        self._emit_bot_update(bot_id)
        self._emit_global_update()

    def record_order_cancel(self, bot_id: int) -> None:
        bot = self._get_bot(bot_id)
        bot.orders_cancelled += 1
        self._emit_bot_update(bot_id)
        self._emit_global_update()

    def record_order_history(
        self,
        source: str,
        owner: str,
        symbol: str,
        side: str,
        order_type: str,
        price: float,
        qty: float,
        filled_pct: float,
        status: str,
        timestamp: float,
        bot_name: str = "",
        pnl: float = 0.0,
        pnl_pct: float = 0.0,
    ) -> None:
        self.order_history.append({
            "source": source,
            "owner": owner,
            "symbol": symbol,
            "side": side,
            "order_type": order_type,
            "price": float(price),
            "qty": float(qty),
            "filled_pct": float(filled_pct),
            "status": status,
            "timestamp": float(timestamp),
            "bot_name": bot_name,
            "pnl": float(pnl),
            "pnl_pct": float(pnl_pct),
        })
        self._emit_global_update()

    def get_order_history(self) -> List[dict]:
        return list(self.order_history)

    def open_trade(
        self,
        bot_id: int,
        order_id: int,
        side: str,
        entry_price: float,
        entry_qty: float,
        entry_time: Optional[float] = None,
    ) -> Trade:
        bot = self._get_bot(bot_id)

        if order_id in bot.open_trades:
            raise ValueError(f"order_id {order_id} already exists in open trades")

        if entry_time is None:
            entry_time = time.time()

        trade = Trade(
            bot_id=bot.bot_id,
            bot_name=bot.bot_name,
            symbol=bot.symbol,
            order_id=int(order_id),
            side=side.upper(),
            entry_price=float(entry_price),
            entry_time=float(entry_time),
            entry_qty=float(entry_qty),
        )

        bot.open_trades[trade.order_id] = trade
        bot.all_trades.append(trade)
        self.order_to_bot[trade.order_id] = bot_id

        signed_qty = trade.direction * trade.entry_qty
        bot.current_position += signed_qty
        bot.position_history.append((trade.entry_time, bot.current_position))

        self._recalculate_unrealized_for_bot(bot_id, timestamp=trade.entry_time)
        self._recalculate_account_equity(timestamp=trade.entry_time)

        payload = self._trade_to_dict(trade)
        self.trade_opened.emit(payload)
        self._emit_bot_update(bot_id)
        self._emit_global_update()
        return trade

    def close_trade(
        self,
        order_id: int,
        exit_price: float,
        exit_qty: Optional[float] = None,
        exit_time: Optional[float] = None,
    ):
        bot_id = self.order_to_bot.get(order_id)
        if bot_id is None:
            raise ValueError(f"Unknown order_id {order_id}")

        bot = self._get_bot(bot_id)
        trade = bot.open_trades.get(order_id)
        if trade is None:
            raise ValueError(f"Open trade not found for order_id {order_id}")

        if exit_time is None:
            exit_time = time.time()

        exit_price = float(exit_price)
        exit_time = float(exit_time)

        remaining_before = trade.remaining_qty
        trade.close(exit_price=exit_price, exit_time=exit_time, exit_qty=exit_qty)

        closed_qty_now = remaining_before if exit_qty is None else float(exit_qty)

        if trade.side.upper() == "BUY":
            realized_piece = (exit_price - trade.entry_price) * closed_qty_now
        else:
            realized_piece = (trade.entry_price - exit_price) * closed_qty_now

        bot.total_realized_pnl += realized_piece
        self.account_realized_pnl += realized_piece

        self.account_cash_balance = self.account_capital_base() + self.account_realized_pnl
        self.account_balance_history.append((exit_time, self.account_cash_balance))

        bot_balance = bot.allocated_balance + bot.total_realized_pnl
        bot.balance_history.append((exit_time, bot_balance))

        bot.current_position -= trade.direction * closed_qty_now
        bot.position_history.append((exit_time, bot.current_position))

        closed_payload = self._trade_to_dict(trade)

        if trade.remaining_qty <= 1e-9:
            bot.closed_trades.append(replace(trade))
            del bot.open_trades[order_id]
            del self.order_to_bot[order_id]

        self._recalculate_unrealized_for_bot(bot_id, timestamp=exit_time)
        self._recalculate_account_equity(timestamp=exit_time)
        self._update_account_drawdown()

        self.trade_closed.emit(closed_payload)
        self._emit_bot_update(bot_id)
        self._emit_global_update()

        return trade, realized_piece

    def update_mark_price(
        self,
        bot_id: int,
        mark_price: float,
        timestamp: Optional[float] = None,
    ) -> None:
        bot = self._get_bot(bot_id)

        if timestamp is None:
            timestamp = time.time()

        bot.mark_price = float(mark_price)
        self._recalculate_unrealized_for_bot(bot_id, timestamp=float(timestamp))
        self._recalculate_account_equity(timestamp=float(timestamp))
        self._update_account_drawdown()

        self._emit_bot_update(bot_id)
        self._emit_global_update()

    def update_symbol_price(
        self,
        symbol: str,
        mark_price: float,
        timestamp: Optional[float] = None,
    ) -> None:
        if timestamp is None:
            timestamp = time.time()

        for bot_id, bot in self.bots.items():
            if bot.symbol == symbol:
                bot.mark_price = float(mark_price)
                self._recalculate_unrealized_for_bot(bot_id, timestamp=float(timestamp))

        self._recalculate_account_equity(timestamp=float(timestamp))
        self._update_account_drawdown()

        for bot_id, bot in self.bots.items():
            if bot.symbol == symbol:
                self._emit_bot_update(bot_id)

        self._emit_global_update()

    def _recalculate_unrealized_for_bot(
        self,
        bot_id: int,
        timestamp: Optional[float] = None,
    ) -> None:
        bot = self._get_bot(bot_id)

        if timestamp is None:
            timestamp = time.time()

        unrealized = 0.0
        if bot.mark_price > 0:
            for trade in bot.open_trades.values():
                unrealized += trade.unrealized_pnl(bot.mark_price)

        bot.total_unrealized_pnl = unrealized

        bot_equity = bot.allocated_balance + bot.total_realized_pnl + bot.total_unrealized_pnl
        bot.equity_history.append((float(timestamp), bot_equity))

    def _recalculate_account_equity(self, timestamp: Optional[float] = None) -> None:
        if timestamp is None:
            timestamp = time.time()

        total_unrealized = 0.0
        for bot in self.bots.values():
            total_unrealized += bot.total_unrealized_pnl

        self.account_unrealized_pnl = total_unrealized
        self.account_equity = self.account_cash_balance + self.account_unrealized_pnl
        self.account_equity_history.append((float(timestamp), self.account_equity))

    def _update_account_drawdown(self) -> None:
        if self.account_equity > self.account_peak_equity:
            self.account_peak_equity = self.account_equity

        if self.account_cash_balance > self.account_peak_cash:
            self.account_peak_cash = self.account_cash_balance

        if self.account_peak_equity > 0:
            self.account_current_drawdown_pct = (
                (self.account_peak_equity - self.account_equity) / self.account_peak_equity
            ) * 100.0
            self.account_max_drawdown_pct = max(
                self.account_max_drawdown_pct,
                self.account_current_drawdown_pct,
            )

    def get_account_summary(self) -> dict:
        all_closed: List[Trade] = []
        all_open: List[Trade] = []
        total_submitted = 0
        total_filled = 0
        total_cancelled = 0
        total_volume = 0.0

        for bot in self.bots.values():
            all_closed.extend(bot.closed_trades)
            all_open.extend(bot.open_trades.values())
            total_submitted += bot.orders_submitted
            total_filled += bot.orders_filled
            total_cancelled += bot.orders_cancelled
            total_volume += bot.total_volume

        capital_base = self.account_capital_base()
        total_pnl = self.account_realized_pnl + self.account_unrealized_pnl

        return {
            "scope": "account",
            "starting_balance": self.starting_balance,
            "net_deposits": self.net_deposits,
            "capital_base": capital_base,
            "cash_balance": self.account_cash_balance,
            "equity": self.account_equity,
            "realized_pnl": self.account_realized_pnl,
            "unrealized_pnl": self.account_unrealized_pnl,
            "total_pnl": total_pnl,
            "pnl_pct": self._safe_pct(total_pnl, capital_base),
            "equity_pct": self._safe_pct(self.account_equity - capital_base, capital_base),
            "orders_submitted": total_submitted,
            "orders_filled": total_filled,
            "orders_cancelled": total_cancelled,
            "fill_rate": self._fill_rate(total_filled, total_submitted),
            "total_volume": total_volume,
            "open_trades": len(all_open),
            "closed_trades": len(all_closed),
            "winning_trades": len([t for t in all_closed if t.pnl > 0]),
            "losing_trades": len([t for t in all_closed if t.pnl < 0]),
            "avg_trade": self._avg_trade(all_closed),
            "avg_winning_trade": self._avg_winning_trade(all_closed),
            "avg_losing_trade": self._avg_losing_trade(all_closed),
            "largest_win": self._largest_win(all_closed),
            "largest_loss": self._largest_loss(all_closed),
            "profit_factor": self._profit_factor(all_closed),
            "max_drawdown_pct": self.account_max_drawdown_pct,
            "current_drawdown_pct": self.account_current_drawdown_pct,
            "sharpe_ratio": self._sharpe_ratio(self.account_equity_history),
            "sortino_ratio": self._sortino_ratio(self.account_equity_history),
            "time_run_seconds": max(0.0, time.time() - self.start_time),
            "balance_history": self.account_balance_history[:],
            "equity_history": self.account_equity_history[:],
            "trades": [self._trade_to_dict(t) for t in all_closed],
            "open_trade_details": [self._trade_to_dict(t) for t in all_open],
            "order_history": self.get_order_history(),
        }

    def get_bot_summary(self, bot_id: int) -> dict:
        bot = self._get_bot(bot_id)

        bot_balance = bot.allocated_balance + bot.total_realized_pnl
        bot_equity = bot_balance + bot.total_unrealized_pnl
        total_pnl = bot.total_realized_pnl + bot.total_unrealized_pnl

        closed = bot.closed_trades[:]
        open_trades = list(bot.open_trades.values())

        bot_order_history = [
            rec
            for rec in self.order_history
            if rec.get("bot_name") == bot.bot_name or rec.get("owner") == bot.bot_name
        ]

        return {
            "scope": "bot",
            "bot_id": bot.bot_id,
            "bot_name": bot.bot_name,
            "symbol": bot.symbol,
            "strategy_name": bot.strategy_name,
            "allocated_balance": bot.allocated_balance,
            "cash_balance": bot_balance,
            "equity": bot_equity,
            "realized_pnl": bot.total_realized_pnl,
            "unrealized_pnl": bot.total_unrealized_pnl,
            "total_pnl": total_pnl,
            "pnl_pct": self._safe_pct(total_pnl, bot.allocated_balance),
            "equity_pct": self._safe_pct(bot_equity - bot.allocated_balance, bot.allocated_balance),
            "orders_submitted": bot.orders_submitted,
            "orders_filled": bot.orders_filled,
            "orders_cancelled": bot.orders_cancelled,
            "fill_rate": self._fill_rate(bot.orders_filled, bot.orders_submitted),
            "total_volume": bot.total_volume,
            "current_position": bot.current_position,
            "mark_price": bot.mark_price,
            "open_trades": len(open_trades),
            "closed_trades": len(closed),
            "winning_trades": len([t for t in closed if t.pnl > 0]),
            "losing_trades": len([t for t in closed if t.pnl < 0]),
            "avg_trade": self._avg_trade(closed),
            "avg_winning_trade": self._avg_winning_trade(closed),
            "avg_losing_trade": self._avg_losing_trade(closed),
            "largest_win": self._largest_win(closed),
            "largest_loss": self._largest_loss(closed),
            "profit_factor": self._profit_factor(closed),
            "max_drawdown_pct": self._max_drawdown_pct(bot.equity_history),
            "sharpe_ratio": self._sharpe_ratio(bot.equity_history),
            "sortino_ratio": self._sortino_ratio(bot.equity_history),
            "time_run_seconds": max(0.0, time.time() - bot.start_time),
            "balance_history": bot.balance_history[:],
            "equity_history": bot.equity_history[:],
            "position_history": bot.position_history[:],
            "trades": [self._trade_to_dict(t) for t in closed],
            "open_trade_details": [self._trade_to_dict(t) for t in open_trades],
            "order_history": bot_order_history,
        }

    def get_all_bot_summaries(self) -> List[dict]:
        return [self.get_bot_summary(bot_id) for bot_id in self.bots.keys()]

    def bot_names(self) -> List[str]:
        return [bot.bot_name for bot in self.bots.values()]

    def account_capital_base(self) -> float:
        return self.starting_balance + self.net_deposits

    def add_account_balance(self, amount: float, timestamp: Optional[float] = None) -> None:
        amount = float(amount)
        if amount <= 0:
            raise ValueError("amount must be > 0")

        if timestamp is None:
            timestamp = time.time()

        self.net_deposits += amount
        self.account_cash_balance += amount
        self.account_equity = self.account_cash_balance + self.account_unrealized_pnl

        self.account_balance_history.append((float(timestamp), self.account_cash_balance))
        self.account_equity_history.append((float(timestamp), self.account_equity))

        if self.account_cash_balance > self.account_peak_cash:
            self.account_peak_cash = self.account_cash_balance
        if self.account_equity > self.account_peak_equity:
            self.account_peak_equity = self.account_equity

        self._update_account_drawdown()
        self._emit_global_update()

    def _emit_global_update(self) -> None:
        self.performance_updated.emit(self.get_account_summary())

    def _emit_bot_update(self, bot_id: int) -> None:
        self.bot_updated.emit(bot_id, self.get_bot_summary(bot_id))

    def _get_bot(self, bot_id: int) -> BotStats:
        if bot_id not in self.bots:
            raise ValueError(f"Bot {bot_id} is not registered")
        return self.bots[bot_id]

    def _trade_to_dict(self, trade: Trade) -> dict:
        bot = self.bots.get(trade.bot_id)
        mark_price = float(bot.mark_price) if bot is not None else 0.0
        unrealized = trade.unrealized_pnl(mark_price) if mark_price > 0 else 0.0

        return {
            "bot_id": trade.bot_id,
            "bot_name": trade.bot_name,
            "strategy_name": bot.strategy_name if bot is not None else "Unknown",
            "symbol": trade.symbol,
            "order_id": trade.order_id,
            "side": trade.side,
            "entry_price": float(trade.entry_price),
            "entry_time": float(trade.entry_time),
            "entry_qty": float(trade.entry_qty),
            "exit_price": float(trade.exit_price),
            "exit_time": float(trade.exit_time),
            "exit_qty": float(trade.exit_qty),
            "remaining_qty": float(trade.remaining_qty),
            "pnl": float(trade.pnl),
            "pnl_pct": float(trade.pnl_pct),
            "unrealized_pnl": float(unrealized),
            "is_closed": trade.is_closed,
        }

    def _safe_pct(self, value: float, base: float) -> float:
        if base == 0:
            return 0.0
        return (value / base) * 100.0

    def _fill_rate(self, fills: int, submitted: int) -> float:
        if submitted == 0:
            return 0.0
        return fills / submitted

    def _avg_trade(self, trades: List[Trade]) -> float:
        if not trades:
            return 0.0
        return sum(t.pnl for t in trades) / len(trades)

    def _avg_winning_trade(self, trades: List[Trade]) -> float:
        wins = [t.pnl for t in trades if t.pnl > 0]
        if not wins:
            return 0.0
        return sum(wins) / len(wins)

    def _avg_losing_trade(self, trades: List[Trade]) -> float:
        losses = [t.pnl for t in trades if t.pnl < 0]
        if not losses:
            return 0.0
        return sum(losses) / len(losses)

    def _largest_win(self, trades: List[Trade]) -> float:
        wins = [t.pnl for t in trades if t.pnl > 0]
        return max(wins, default=0.0)

    def _largest_loss(self, trades: List[Trade]) -> float:
        losses = [t.pnl for t in trades if t.pnl < 0]
        return min(losses, default=0.0)

    def _profit_factor(self, trades: List[Trade]) -> float:
        gross_profit = sum(t.pnl for t in trades if t.pnl > 0)
        gross_loss = abs(sum(t.pnl for t in trades if t.pnl < 0))
        if gross_loss == 0:
            return gross_profit if gross_profit > 0 else 0.0
        return gross_profit / gross_loss

    def _returns_series(self, equity_history: List[Tuple[float, float]]) -> List[float]:
        if len(equity_history) < 2:
            return []

        returns: List[float] = []
        for i in range(1, len(equity_history)):
            prev_equity = float(equity_history[i - 1][1])
            curr_equity = float(equity_history[i][1])

            if prev_equity != 0:
                returns.append((curr_equity - prev_equity) / prev_equity)

        return returns

    def _sharpe_ratio(self, equity_history: List[Tuple[float, float]]) -> float:
        returns = self._returns_series(equity_history)
        if len(returns) < 2:
            return 0.0

        mean_ret = sum(returns) / len(returns)
        variance = sum((r - mean_ret) ** 2 for r in returns) / (len(returns) - 1)
        std = math.sqrt(variance)

        if std == 0:
            return 0.0
        return mean_ret / std

    def _sortino_ratio(self, equity_history: List[Tuple[float, float]]) -> float:
        returns = self._returns_series(equity_history)
        if len(returns) < 2:
            return 0.0

        mean_ret = sum(returns) / len(returns)
        downside = [r for r in returns if r < 0]
        if not downside:
            return 0.0

        downside_var = sum(r ** 2 for r in downside) / len(downside)
        downside_std = math.sqrt(downside_var)

        if downside_std == 0:
            return 0.0
        return mean_ret / downside_std

    def _max_drawdown_pct(self, equity_history: List[Tuple[float, float]]) -> float:
        if not equity_history:
            return 0.0

        peak = float(equity_history[0][1])
        max_dd = 0.0

        for _, equity in equity_history:
            equity = float(equity)
            if equity > peak:
                peak = equity

            if peak > 0:
                dd = ((peak - equity) / peak) * 100.0
                max_dd = max(max_dd, dd)

        return max_dd