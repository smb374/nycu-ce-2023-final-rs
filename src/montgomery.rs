use rug::Integer;

struct Montgomery {
    n: Integer,
    n_neg_inv: Integer,
    r: Integer,
    r2: Integer,
    r_mask: Integer,
    bits: usize,
}
