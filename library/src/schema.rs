table! {
    readings (id) {
        id -> Integer,
        publisher_id -> BigInt,
        eco2 -> Integer,
        evtoc -> Integer,
        read_time -> BigInt,
        start_time -> BigInt,
        increment -> Text,
    }
}
