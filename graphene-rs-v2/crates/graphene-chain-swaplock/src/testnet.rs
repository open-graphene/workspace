/// Public Swaplock testnet database RPC endpoint used by examples and live tests.
pub const SWAPLOCK_TESTNET_HTTP_URL: &str = "https://node01.swaplock.chainpool.online:8090";

/// Public Swaplock testnet WebSocket endpoint used for `network_broadcast_api`.
pub const SWAPLOCK_TESTNET_WS_URL: &str = "wss://node01.swaplock.chainpool.online:8090";

/// Public testnet-only WIF for the shared `swaplock` account.
///
/// This key is intentionally shared for Swaplock testnet development. Never copy
/// this pattern for mainnet keys or user-controlled funds.
pub const PUBLIC_SWAPLOCK_TESTNET_WIF: &str = "5K71C3PVyynjDdzxNdgd5YJ6y8Z86eEc5RNPvAN983UdhCF4HPw";

/// Shared Swaplock testnet sender account used by examples and live tests.
pub const SWAPLOCK_TESTNET_FROM_ACCOUNT: &str = "swaplock";

/// Shared Swaplock testnet recipient account used by examples and live tests.
pub const SWAPLOCK_TESTNET_TO_ACCOUNT: &str = "committee-account";
