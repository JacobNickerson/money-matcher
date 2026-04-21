import importlib.util
import time
import random
from dataclasses import dataclass
from collections import deque
from PyQt5.QtCore import QObject, pyqtSignal

@dataclass
class MarketTrade:
    symbol: str
    price: float
    qty: int
    side: str
    timestamp: int

@dataclass
class BookState:
    best_bid: float
    best_ask: float
    spread: float
    mid_price: float

class StrategyRunner(QObject):
    strategy_log = pyqtSignal(str)

    def __init__(self, strategy_file_path, bot_config, order_entry=None, performance_tracker=None):
        super().__init__()

        self.strategy_file_path = strategy_file_path
        self.bot_config = bot_config
        self.order_entry = order_entry
        self.performance_tracker = performance_tracker

        self.bot_id = bot_config["id"]
        self.bot_name = bot_config.get("bot_name", bot_config.get("name", f"bot_{self.bot_id}"))
        self.symbol = bot_config.get("symbol", "")
        self.strategy_name = bot_config.get("strategy_name", "Unknown")

        self.order_size = float(bot_config.get("order_size", 1.0))
        self.max_position = float(bot_config.get("max_position", 100.0))
        self.latency_ms = int(bot_config.get("latency", bot_config.get("latency_ms", 0)))
        self.jitter_ms = int(bot_config.get("jitter", bot_config.get("jitter_ms", 0)))
        self.starting_balance = float(bot_config.get("starting_balance", 10000.0))

        self.strategy = None
        self.status = "Paused"
        self.reserved_cash = 0.0

        self.position = 0.0
        self.book_state = None
        self.last_trade = None
        self.last_fill = None
        self.started_at = None
        self.trades_processed = 0
        self.last_trade_time = None
        self.pending_orders = {}

        # for matching fills to tracked positions
        self.next_synthetic_order_id = 1
        self.open_long_order_ids = deque()   # BUY fills still open
        self.open_short_order_ids = deque()  # SELL fills still open
        self.orders_with_any_fill = set()

        if self.performance_tracker is not None:
            self.performance_tracker.register_bot(
                bot_id=self.bot_id,
                bot_name=self.bot_name,
                symbol=self.symbol,
                allocated_balance=self.starting_balance,
                strategy_name=self.strategy_name
            )

    def load_strategy(self):
        try:
            spec = importlib.util.spec_from_file_location(
                f"strategy_{self.bot_id}",
                self.strategy_file_path
            )

            if spec is None or spec.loader is None:
                raise ValueError(f"Could not load strategy file: {self.strategy_file_path}")

            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)

            strategy_class = None
            for name in dir(module):
                obj = getattr(module, name)
                if (
                    isinstance(obj, type)
                    and hasattr(obj, "on_start")
                    and hasattr(obj, "on_book")
                    and hasattr(obj, "on_trade")
                    and hasattr(obj, "on_fill")
                    and hasattr(obj, "on_timer")
                    and hasattr(obj, "on_stop")
                ):
                    strategy_class = obj
                    break

            if strategy_class is None:
                raise ValueError(f"No valid strategy class found in {self.strategy_file_path}")

            self.strategy = strategy_class()
            self.strategy.bot = self
            self.strategy.bot_name = self.bot_name
            self.strategy.symbol = self.symbol

        except Exception as e:
            self.log(f"Error loading strategy from {self.strategy_file_path}: {e}")
            raise

    def start(self):
        if self.status == "Active":
            return

        if self.strategy is None:
            self.load_strategy()

        self.status = "Active"
        self.started_at = time.time()
        self.log(f"Started strategy for {self.bot_name}")
        self.strategy.on_start()

    def stop(self):
        if self.status != "Active":
            return

        self.status = "Paused"
        self.strategy.on_stop()
        self.log(f"Stopped strategy for {self.bot_name}")

    def on_market_event(self, event, rust_book):
        if self.status != "Active" or self.strategy is None:
            return

        try:
            old_book = self.book_state
            self.on_order_book_update(rust_book)

            # Update tracker mark price from book mid-price
            if (
                self.performance_tracker is not None
                and self.book_state is not None
                and self.book_state.mid_price is not None
            ):
                self.performance_tracker.update_mark_price(
                    bot_id=self.bot_id,
                    mark_price=float(self.book_state.mid_price),
                    timestamp=float(event.timestamp) if hasattr(event, "timestamp") else time.time()
                )

            if self.book_state is not None:
                if (
                    self.book_state.best_bid is not None
                    and self.book_state.best_ask is not None
                    and self.book_state.spread is not None
                ):
                    changed = (
                        old_book is None
                        or old_book.best_bid != self.book_state.best_bid
                        or old_book.best_ask != self.book_state.best_ask
                        or old_book.spread != self.book_state.spread
                        or old_book.mid_price != self.book_state.mid_price
                    )
                    if changed:
                        self.strategy.on_book(self.book_state)

                if type(event.kind).__name__ == "Trade":
                    try:
                        trade = MarketTrade(
                            symbol=self.symbol,
                            price=float(event.kind.price),
                            qty=int(event.kind.quantity),
                            side="BUY" if "Bid" in str(event.kind.aggressor_side) else "SELL",
                            timestamp=int(event.timestamp),
                        )

                        self.trades_processed += 1
                        self.last_trade_time = time.time()
                        self.last_trade = trade

                        # Also update mark price from last traded price
                        if self.performance_tracker is not None:
                            self.performance_tracker.update_mark_price(
                                bot_id=self.bot_id,
                                mark_price=float(trade.price),
                                timestamp=float(trade.timestamp)
                            )

                        self.strategy.on_trade(trade)

                    except (AttributeError, ValueError) as e:
                        print(f"Error parsing trade event: {e}")

        except Exception as e:
            self.log(f"Error handling market event: {e}")

    def on_order_book_update(self, book):
        if self.status != "Active" or self.strategy is None:
            return

        try:
            best_bid = book.best_bid()
            best_ask = book.best_ask()
            spread = book.best_ask() - book.best_bid()
            mid_price = book.mid_price()

            self.book_state = BookState(
                best_bid=best_bid,
                best_ask=best_ask,
                spread=spread,
                mid_price=mid_price
            )

        except Exception as e:
            self.book_state = None
            self.log(f"Error processing order book update: {e}")

    def submit_order(self, side, price, qty):
        if self.status != "Active":
            self.log("Order ignored: strategy is paused")
            return

        try:
            qty = float(qty)
            side = side.upper()

            if side == "BUY":
                if self.position + qty > self.max_position:
                    self.log("BUY blocked: max position exceeded")
                    return

                required_cost = float(price) * float(qty)
                account_summary = self.performance_tracker.get_account_summary()
                cash_balance = float(account_summary.get("cash_balance", 0.0))
                available_cash = cash_balance - self.reserved_cash

                if required_cost > available_cash:
                    self.log(f"BUY blocked: insufficient available funds (need {required_cost:.2f}, have {available_cash:.2f})")
                    return

                self.reserved_cash += required_cost

            if side == "SELL" and self.position - qty < -self.max_position:
                self.log("SELL blocked: max position exceeded")
                return

            if self.performance_tracker is not None:
                self.performance_tracker.record_order_submission(self.bot_id)

            if self.order_entry is not None:
                order_id = self.order_entry.submit_order(
                    bot_order=True,
                    _side=side,
                    _price=price,
                    _qty=qty
                )

                if order_id is not None:
                    now = time.time()
                    self.pending_orders[order_id] = {
                        "order_id": order_id,
                        "symbol": self.symbol,
                        "side": side,
                        "price": price,
                        "qty": qty,
                        "remaining_qty": qty,
                        "submitted_at": now,
                        "order_type": "Limit",
                        "source": "bot",
                        "owner_id": self.bot_id,
                        "status": "Open",
                        "filled_pct": 0.0,
                        "next_fill_check": now + 0.5,
                    }

                self.log(f"Submitted {side} order: qty={qty}, price={price}")

        except Exception as e:
            self.log(f"Order submit error: {e}")

    def cancel_order(self, side, order_id):
        if order_id in self.pending_orders:
            self.pending_orders.pop(order_id, None)

        if self.order_entry is not None and hasattr(self.order_entry, "cancel_order"):
            self.order_entry.cancel_order(side, order_id)

        self.log(f"Cancelled order {order_id}")

    def buy(self, qty=None, price=None):
        qty = self.order_size if qty is None else qty
        if price is None and self.book_state is not None:
            price = self.book_state.best_ask
        self.submit_order("BUY", price, qty)

    def sell(self, qty=None, price=None):
        qty = self.order_size if qty is None else qty
        if price is None and self.book_state is not None:
            price = self.book_state.best_bid
        self.submit_order("SELL", price, qty)

    def on_fill(self, fill):
        if self.status != "Active" or self.strategy is None or fill is None:
            return

        self.last_fill = fill

        try:
            side = str(getattr(fill, "side", "")).upper()
            qty = float(getattr(fill, "qty", 0.0))
            price = float(getattr(fill, "price", 0.0))
            timestamp = float(getattr(fill, "timestamp", time.time()))
            order_id = getattr(fill, "order_id", None)

            if qty <= 0:
                self.log("Ignoring fill with non-positive quantity")
                return

            if self.performance_tracker is not None:
                # always count executed notional volume for every fill
                self.performance_tracker.record_fill_volume(
                    bot_id=self.bot_id,
                    price=price,
                    qty=qty
                )

                # only count the order once toward fill-rate metrics
                if order_id is not None and order_id not in self.orders_with_any_fill:
                    self.performance_tracker.record_order_fill(self.bot_id)
                    self.orders_with_any_fill.add(order_id)

            # Update local runner state first
            if side == "BUY":
                self.position += qty
                self.reserved_cash -= price * qty
                self.reserved_cash = max(0.0, self.reserved_cash)
            elif side == "SELL":
                self.position -= qty

            # Match this fill directly into tracker trade state
            match_result = self.apply_fill_to_tracker(
                side=side,
                price=price,
                qty=qty,
                timestamp=timestamp,
                order_id=order_id
            )

            realized_pnl = 0.0
            realized_qty = 0.0
            realized_pct = 0.0

            if isinstance(match_result, dict):
                realized_pnl = float(match_result.get("realized_pnl", 0.0))
                realized_qty = float(match_result.get("realized_qty", 0.0))

            if realized_qty > 0 and price > 0:
                realized_pct = (realized_pnl / (price * realized_qty)) * 100.0

            if (
                self.performance_tracker is not None
                and order_id is not None
                and order_id in self.pending_orders
            ):
                order = self.pending_orders[order_id]
                remaining_qty = float(order.get("remaining_qty", 0.0))

                if remaining_qty <= 1e-9:
                    self.performance_tracker.record_order_history(
                        source=order.get("source", "Bot"),
                        owner=self.bot_name,
                        symbol=order.get("symbol", self.symbol),
                        side=order.get("side", side),
                        order_type=order.get("order_type", "Limit"),
                        price=price,
                        qty=float(order.get("qty", qty)),
                        filled_pct=float(order.get("filled_pct", 100.0)),
                        status=order.get("status", "Filled"),
                        timestamp=timestamp,
                        bot_name=self.bot_name,
                        pnl=realized_pnl,
                        pnl_pct=realized_pct,
                    )

            self.strategy.on_fill(fill)
            self.log(f"Fill received: {side} {qty}, new position={self.position}")

        except Exception as e:
            self.log(f"Error handling fill: {e}")

    def apply_fill_to_tracker(self, side, price, qty, timestamp, order_id=None):
        if self.performance_tracker is None:
            return {"realized_pnl": 0.0, "realized_qty": 0.0}

        remaining_qty = float(qty)
        realized_pnl_total = 0.0
        realized_qty_total = 0.0

        if side == "BUY":
            # Close shorts first
            while remaining_qty > 0 and self.open_short_order_ids:
                open_order_id = self.open_short_order_ids[0]

                bot_summary = self.performance_tracker.get_bot_summary(self.bot_id)
                open_trade_details = bot_summary.get("open_trade_details", [])
                trade_info = next(
                    (t for t in open_trade_details if t["order_id"] == open_order_id),
                    None
                )

                if trade_info is None:
                    self.open_short_order_ids.popleft()
                    continue

                close_qty = min(remaining_qty, float(trade_info["remaining_qty"]))

                _trade, realized_piece = self.performance_tracker.close_trade(
                    order_id=open_order_id,
                    exit_price=price,
                    exit_qty=close_qty,
                    exit_time=timestamp
                )

                realized_pnl_total += float(realized_piece)
                realized_qty_total += float(close_qty)
                remaining_qty -= close_qty

                updated_summary = self.performance_tracker.get_bot_summary(self.bot_id)
                updated_open = updated_summary.get("open_trade_details", [])
                still_open = next(
                    (t for t in updated_open if t["order_id"] == open_order_id),
                    None
                )

                if still_open is None or float(still_open["remaining_qty"]) <= 0:
                    self.open_short_order_ids.popleft()

            # Open new long trade for leftover qty
            if remaining_qty > 0:
                trade_id = self._next_order_id()
                self.performance_tracker.open_trade(
                    bot_id=self.bot_id,
                    order_id=trade_id,
                    side="BUY",
                    entry_price=price,
                    entry_qty=remaining_qty,
                    entry_time=timestamp
                )
                self.open_long_order_ids.append(trade_id)

        elif side == "SELL":
            # Close longs first
            while remaining_qty > 0 and self.open_long_order_ids:
                open_order_id = self.open_long_order_ids[0]

                bot_summary = self.performance_tracker.get_bot_summary(self.bot_id)
                open_trade_details = bot_summary.get("open_trade_details", [])
                trade_info = next(
                    (t for t in open_trade_details if t["order_id"] == open_order_id),
                    None
                )

                if trade_info is None:
                    self.open_long_order_ids.popleft()
                    continue

                close_qty = min(remaining_qty, float(trade_info["remaining_qty"]))

                _trade, realized_piece = self.performance_tracker.close_trade(
                    order_id=open_order_id,
                    exit_price=price,
                    exit_qty=close_qty,
                    exit_time=timestamp
                )

                realized_pnl_total += float(realized_piece)
                realized_qty_total += float(close_qty)
                remaining_qty -= close_qty

                updated_summary = self.performance_tracker.get_bot_summary(self.bot_id)
                updated_open = updated_summary.get("open_trade_details", [])
                still_open = next(
                    (t for t in updated_open if t["order_id"] == open_order_id),
                    None
                )

                if still_open is None or float(still_open["remaining_qty"]) <= 0:
                    self.open_long_order_ids.popleft()

            # Open new short trade for leftover qty
            if remaining_qty > 0:
                trade_id = self._next_order_id()
                self.performance_tracker.open_trade(
                    bot_id=self.bot_id,
                    order_id=trade_id,
                    side="SELL",
                    entry_price=price,
                    entry_qty=remaining_qty,
                    entry_time=timestamp
                )
                self.open_short_order_ids.append(trade_id)

        return {
            "realized_pnl": realized_pnl_total,
            "realized_qty": realized_qty_total,
        }

    def on_trade(self, trade):
        if self.status != "Active" or self.strategy is None or trade is None:
            return

        self.last_trade = trade

        try:
            if self.performance_tracker is not None:
                self.performance_tracker.update_mark_price(
                    bot_id=self.bot_id,
                    mark_price=float(trade.price),
                    timestamp=float(getattr(trade, "timestamp", time.time()))
                )

            self.strategy.on_trade(trade)
        except Exception as e:
            self.log(f"Error handling trade: {e}")

    def on_timer(self, now=None):
        if self.status != "Active" or self.strategy is None:
            return

        if now is None:
            now = time.time()

        try:
            self.strategy.on_timer(now)
        except Exception as e:
            self.log(f"Error handling timer: {e}")

    def _next_order_id(self):
        oid = self.next_synthetic_order_id
        self.next_synthetic_order_id += 1
        return oid

    def get_stats(self):
        stats = {
            "bot_id": self.bot_id,
            "bot_name": self.bot_name,
            "symbol": self.symbol,
            "strategy_name": self.bot_config.get("strategy_name", "Unknown strategy"),
            "status": self.status,
            "position": self.position,
            "order_size": self.order_size,
            "max_position": self.max_position,
            "latency_ms": self.latency_ms,
            "jitter_ms": self.jitter_ms,
            "last_trade_price": getattr(self.last_trade, "price", None) if self.last_trade else None,
            "best_bid": self.book_state.best_bid if self.book_state else None,
            "best_ask": self.book_state.best_ask if self.book_state else None,
            "spread": self.book_state.spread if self.book_state else None,
            "mid_price": self.book_state.mid_price if self.book_state else None,
            "pending_orders": len(self.pending_orders),
        }

        if self.performance_tracker is not None:
            try:
                summary = self.performance_tracker.get_bot_summary(self.bot_id)

                realized = float(summary.get("realized_pnl", 0.0))
                unrealized = float(summary.get("unrealized_pnl", 0.0))
                allocated = float(summary.get("allocated_balance", 0.0))
                total_pnl = realized + unrealized
                total_pnl_pct = (total_pnl / allocated * 100.0) if allocated else 0.0

                stats.update({
                    "cash_balance": summary.get("cash_balance", 0.0),
                    "equity": summary.get("equity", 0.0),
                    "realized_pnl": realized,
                    "unrealized_pnl": unrealized,
                    "total_pnl": total_pnl,
                    "pnl_pct": total_pnl_pct,
                    "equity_pct": summary.get("equity_pct", 0.0),
                    "orders_submitted": summary.get("orders_submitted", 0),
                    "orders_filled": summary.get("orders_filled", 0),
                    "orders_cancelled": summary.get("orders_cancelled", 0),
                    "fill_rate": summary.get("fill_rate", 0.0),
                    "total_volume": summary.get("total_volume", 0.0),
                    "position": summary.get("current_position", self.position),
                    "runner_position": self.position,
                    "mark_price": summary.get("mark_price", 0.0),
                    "closed_trades": summary.get("closed_trades", 0),
                    "winning_trades": summary.get("winning_trades", 0),
                    "losing_trades": summary.get("losing_trades", 0),
                    "avg_trade": summary.get("avg_trade", 0.0),
                    "profit_factor": summary.get("profit_factor", 0.0),
                    "max_drawdown_pct": summary.get("max_drawdown_pct", 0.0),
                })
            except Exception as e:
                self.log(f"Error getting tracker summary: {e}")

        return stats

    def log(self, message):
        self.strategy_log.emit(f"[{self.bot_name}] {message}")

