/// Public Acta testnet database RPC endpoint used by examples and live tests.
pub const ACTA_TESTNET_HTTP_URL: &str = "https://node01.acta.chainpool.online:8090";

/// Public Acta testnet WebSocket endpoint used for `network_broadcast_api`.
pub const ACTA_TESTNET_WS_URL: &str = "wss://node01.acta.chainpool.online:8090";

/// Public testnet-only WIF for the shared Acta testnet account.
///
/// This key is intentionally shared for Acta testnet development. Never copy
/// this pattern for mainnet keys or user-controlled funds.
pub const PUBLIC_ACTA_TESTNET_WIF: &str = "5JBuxPjk1zs8bfKzr9soUDaJGstkA7mZtxz67uC7KeW1CzZBVyZ";

/// Shared Acta testnet sender account used by examples and live tests.
pub const ACTA_TESTNET_FROM_ACCOUNT: &str = "actanet-root";

/// Shared Acta testnet recipient account used by examples and live tests.
pub const ACTA_TESTNET_TO_ACCOUNT: &str = "committee-account";

/// Acta testnet asset used by transfer examples and live tests.
pub const ACTA_TESTNET_TRANSFER_ASSET: &str = "1.3.0";

/// Tiny transfer amount used by live tests and examples.
pub const ACTA_TESTNET_TRANSFER_AMOUNT: i64 = 1;
