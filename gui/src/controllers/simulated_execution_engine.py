import random
import time
from dataclasses import dataclass

@dataclass
class Fill:
    order_id: int
    symbol: str
    side: str
    qty: float
    price: float
    timestamp: float

class SimulatedExecutionEngine:
    def __init__(
        self,
        min_check_delay=0.20,
        max_check_delay=1.00,
        min_fill_ratio=0.25,
        max_fill_ratio=1.00,
        min_fill_qty=0.10,
        market_slippage_bps=10.0,
    ):
        self.min_check_delay = float(min_check_delay)
        self.max_check_delay = float(max_check_delay)
        self.min_fill_ratio = float(min_fill_ratio)
        self.max_fill_ratio = float(max_fill_ratio)
        self.min_fill_qty = float(min_fill_qty)
        self.market_slippage_bps = float(market_slippage_bps)

    def process_runner(self, runner, now=None):
        if now is None:
            now = time.time()

        fills = []

        pending_orders = getattr(runner, "pending_orders", {})
        book_state = getattr(runner, "book_state", None)

        for order in list(pending_orders.values()):
            if now < float(order.get("next_fill_check", 0.0)):
                continue

            fill = self.build_fill(order, book_state, now)
            if fill is None:
                order["next_fill_check"] = now + self._next_delay()
                continue

            fills.append(fill)

            self._apply_fill_to_order(order, fill)
            runner.on_fill(fill)

            if order["remaining_qty"] <= 1e-9:
                if runner.performance_tracker is not None:
                    runner.performance_tracker.record_order_history(
                        source="Bot",
                        owner=runner.bot_name,
                        symbol=order.get("symbol", ""),
                        side=order.get("side", ""),
                        order_type=order.get("order_type", "Limit"),
                        price=float(fill.price),
                        qty=float(order.get("qty", 0.0)),
                        filled_pct=100.0,
                        status="Filled",
                        timestamp=float(fill.timestamp),
                        bot_name=runner.bot_name,
                        pnl=realized_pnl,
                        pnl_pct=realized_pct,
                    )

                pending_orders.pop(order["order_id"], None)
            else:
                order["next_fill_check"] = now + self._next_delay()

        return fills

    def process_manual_orders(self, order_entry, rust_book, now=None, on_fill=None):
        if now is None:
            now = time.time()

        fills = []

        manual_orders = getattr(order_entry, "manual_open_orders", {})

        for order in list(manual_orders.values()):
            if now < float(order.get("next_fill_check", 0.0)):
                continue

            fill = self.build_fill(order, rust_book, now, use_rust_book=True)
            if fill is None:
                order["next_fill_check"] = now + self._next_delay()
                continue

            fills.append(fill)

            if callable(on_fill):
                on_fill(fill)

            self._apply_fill_to_manual_order(order_entry, order, fill)

        return fills

    def build_fill(self, order, book, now, use_rust_book=False):
        if not self._is_order_marketable(order, book, use_rust_book=use_rust_book):
            return None

        fill_qty = self._simulated_fill_qty(order)
        if fill_qty <= 0:
            return None

        fill_price = self._simulated_fill_price(order, book, use_rust_book=use_rust_book)

        return Fill(
            order_id=int(order["order_id"]),
            symbol=str(order["symbol"]),
            side=str(order["side"]).upper(),
            qty=float(fill_qty),
            price=float(fill_price),
            timestamp=float(now),
        )

    def _is_order_marketable(self, order, book, use_rust_book=False):
        side = str(order.get("side", "")).upper()
        order_type = str(order.get("order_type", "Limit")).title()

        if order_type == "Market":
            return True

        limit_price = float(order.get("price", 0.0))
        best_bid, best_ask = self._best_bid_ask(book, use_rust_book=use_rust_book)

        if side == "BUY":
            return best_ask is not None and limit_price >= best_ask
        elif side == "SELL":
            return best_bid is not None and limit_price <= best_bid

        return False

    def _simulated_fill_qty(self, order):
        remaining = float(order.get("remaining_qty", order.get("qty", 0.0)))
        if remaining <= 0:
            return 0.0

        if remaining <= self.min_fill_qty:
            return remaining

        ratio = random.uniform(self.min_fill_ratio, self.max_fill_ratio)
        qty = remaining * ratio
        qty = max(self.min_fill_qty, qty)

        return min(remaining, qty)

    def _simulated_fill_price(self, order, book, use_rust_book=False):
        side = str(order.get("side", "")).upper()
        order_type = str(order.get("order_type", "Limit")).title()
        limit_price = float(order.get("price", 0.0))

        best_bid, best_ask = self._best_bid_ask(book, use_rust_book=use_rust_book)

        if side == "BUY":
            if best_ask is None:
                return limit_price

            if order_type == "Market":
                return best_ask * (1.0 + self.market_slippage_bps / 10000.0)

            return min(limit_price, best_ask)

        if side == "SELL":
            if best_bid is None:
                return limit_price

            if order_type == "Market":
                return best_bid * (1.0 - self.market_slippage_bps / 10000.0)

            return max(limit_price, best_bid)

        return limit_price

    def _best_bid_ask(self, book, use_rust_book=False):
        if book is None:
            return None, None

        if use_rust_book:
            try:
                best_bid = book.best_bid()
            except BaseException:
                best_bid = None

            try:
                best_ask = book.best_ask()
            except BaseException:
                best_ask = None

            return (
                float(best_bid) if best_bid is not None else None,
                float(best_ask) if best_ask is not None else None,
            )

        best_bid = getattr(book, "best_bid", None)
        best_ask = getattr(book, "best_ask", None)

        if callable(best_bid):
            try:
                best_bid = best_bid()
            except BaseException:
                best_bid = None

        if callable(best_ask):
            try:
                best_ask = best_ask()
            except BaseException:
                best_ask = None

        return (
            float(best_bid) if best_bid is not None else None,
            float(best_ask) if best_ask is not None else None,
        )

    def _apply_fill_to_order(self, order, fill):
        original_qty = float(order.get("qty", 0.0))
        remaining_qty = float(order.get("remaining_qty", original_qty))

        remaining_qty = max(0.0, remaining_qty - float(fill.qty))
        order["remaining_qty"] = remaining_qty

        filled_qty = original_qty - remaining_qty
        order["filled_pct"] = 0.0 if original_qty <= 0 else (filled_qty / original_qty) * 100.0
        order["status"] = "Filled" if remaining_qty <= 1e-9 else "Partial"

    def _apply_fill_to_manual_order(self, order_entry, order, fill):
        self._apply_fill_to_order(order, fill)

        order_id = int(order["order_id"])
        remaining_qty = float(order["remaining_qty"])

        if remaining_qty <= 1e-9:
            completed_order = dict(order)
            completed_order["status"] = "Filled"
            completed_order["filled_pct"] = 100.0
            completed_order["filled_at"] = float(fill.timestamp)
            completed_order["fill_price"] = float(fill.price)
            completed_order["fill_qty"] = float(fill.qty)

            if not hasattr(order_entry, "manual_order_history"):
                order_entry.manual_order_history = []

            order_entry.manual_order_history.append(completed_order)

            order_entry.manual_open_orders.pop(order_id, None)
            order_entry.all_open_orders.pop(order_id, None)
        else:
            order["next_fill_check"] = time.time() + self._next_delay()
            order_entry.manual_open_orders[order_id] = order
            order_entry.all_open_orders[order_id] = order

    def _next_delay(self):
        return random.uniform(self.min_check_delay, self.max_check_delay)