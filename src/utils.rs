
pub fn join<TCollection, TItem, TFunc>(
    iter : TCollection,
    separator : &str,
    fmt : TFunc,
) -> String
    where
        TCollection : IntoIterator<Item = TItem>,
        TFunc : Fn(TItem) -> String
{
    iter.into_iter().map( |i| fmt(i) ).collect::<Vec<_>>().join( separator )
}
