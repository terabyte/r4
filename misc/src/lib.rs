#[derive(Clone)]
pub enum Either<L, R> {
    Right(R),
    Left(L),
}

impl<L, R> Either<L, R> {
    pub fn convert_r_mut<F: FnOnce(&L) -> R>(&mut self, f: F) -> &mut R {
        // Arggh, I wish f could take L...
        let new_r;
        match self {
            Either::Right(r) => {
                return r;
            }
            Either::Left(l) => {
                new_r = f(l);
            }
        }
        *self = Either::Right(new_r);
        match self {
            Either::Right(r) => {
                return r;
            }
            Either::Left(_l) => {
                unreachable!();
            }
        }
    }

    pub fn map_left<L2, F: FnOnce(L) -> L2>(self, f: F) -> Either<L2, R> {
        return match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        };
    }
}
