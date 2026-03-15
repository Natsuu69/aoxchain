/// Logical column families used by the persistent data layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColumnFamily {
    Blocks,
    Transactions,
    Receipts,
    State,
}

impl ColumnFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blocks => "blocks",
            Self::Transactions => "transactions",
            Self::Receipts => "receipts",
            Self::State => "state",
        }
    }
}

pub fn all_column_families() -> [ColumnFamily; 4] {
    [
        ColumnFamily::Blocks,
        ColumnFamily::Transactions,
        ColumnFamily::Receipts,
        ColumnFamily::State,
    ]
}
