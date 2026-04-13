use tessera_components::text::text;
use tessera_shard::shard;

#[shard]
pub fn home() {
    text().content("This is the home page.");
}
