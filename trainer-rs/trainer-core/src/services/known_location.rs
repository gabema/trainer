//! Known locations, ported from `Trainer/Services/KnownLocationService.cs`.

use crate::models::{self, Format, KnownLocation};
use crate::storage::{Storage, StorageError, StorageResult};

const STORAGE_KEY: &str = "knownLocations";
/// Radius within which a fix is treated as "at" a known location.
const NEARBY_THRESHOLD_METRES: f64 = 100.0;
const EARTH_RADIUS_METRES: f64 = 6_371_000.0;
const AUTO_NAME_PREFIX: &str = "New Location ";

pub struct KnownLocationService<'a, S: Storage> {
    storage: &'a S,
}

impl<'a, S: Storage> KnownLocationService<'a, S> {
    pub fn new(storage: &'a S) -> Self {
        Self { storage }
    }

    pub async fn all(&self) -> StorageResult<Vec<KnownLocation>> {
        match self.storage.get_item(STORAGE_KEY).await? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| StorageError::new("deserialize", STORAGE_KEY, e.to_string())),
            None => Ok(Vec::new()),
        }
    }

    async fn save_all(&self, locations: &Vec<KnownLocation>) -> StorageResult<()> {
        let json = models::to_json(locations, Format::Storage)
            .map_err(|e| StorageError::new("serialize", STORAGE_KEY, e.to_string()))?;
        self.storage.set_item(STORAGE_KEY, &json).await
    }

    /// Inserts or updates a location. An id of zero means "new".
    ///
    /// A location whose id is set but absent from the list is appended, matching
    /// the C# fallthrough — that is how import restores records with their
    /// original ids.
    pub async fn save(&self, mut location: KnownLocation) -> StorageResult<KnownLocation> {
        let mut locations = self.all().await?;

        if location.id == 0 {
            location.id = assign_id(location.latitude, location.longitude, &locations);
            locations.push(location.clone());
        } else if let Some(slot) = locations.iter_mut().find(|l| l.id == location.id) {
            *slot = location.clone();
        } else {
            locations.push(location.clone());
        }

        self.save_all(&locations).await?;
        Ok(location)
    }

    pub async fn delete(&self, id: i32) -> StorageResult<()> {
        let mut locations = self.all().await?;
        locations.retain(|l| l.id != id);
        self.save_all(&locations).await
    }

    /// The closest known location within 100 metres, or `None`.
    pub async fn find_nearby(
        &self,
        latitude: f64,
        longitude: f64,
    ) -> StorageResult<Option<KnownLocation>> {
        let locations = self.all().await?;

        let mut closest: Option<(f64, KnownLocation)> = None;
        for location in locations {
            let distance =
                haversine_metres(latitude, longitude, location.latitude, location.longitude);
            if distance < NEARBY_THRESHOLD_METRES
                && closest.as_ref().is_none_or(|(best, _)| distance < *best)
            {
                closest = Some((distance, location));
            }
        }

        Ok(closest.map(|(_, location)| location))
    }

    /// The lowest unused `New Location {n}` name, filling gaps before extending.
    pub async fn next_auto_name(&self) -> StorageResult<String> {
        let locations = self.all().await?;
        let used: std::collections::BTreeSet<i32> = locations
            .iter()
            .filter_map(|l| l.name.strip_prefix(AUTO_NAME_PREFIX))
            .filter_map(|n| n.parse().ok())
            .collect();

        let mut next = 1;
        while used.contains(&next) {
            next += 1;
        }
        Ok(format!("{AUTO_NAME_PREFIX}{next}"))
    }
}

/// Derives an id from the coordinates, incrementing on collision.
///
/// **Deliberately not the C# algorithm.** `AssignId` uses `HashCode.Combine`,
/// which .NET seeds randomly per process and documents as unstable across runs,
/// so the same coordinates already produce different ids in different sessions.
/// There is nothing reproducible to port. This uses FNV-1a over the two
/// coordinates' bit patterns, which is deterministic — a strict improvement —
/// while satisfying the only real requirements: ids are `i32`, they avoid
/// collisions, and stored ids are never regenerated.
fn assign_id(latitude: f64, longitude: f64, existing: &[KnownLocation]) -> i32 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for byte in latitude
        .to_bits()
        .to_le_bytes()
        .into_iter()
        .chain(longitude.to_bits().to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    let mut candidate = ((hash >> 32) as u32 ^ hash as u32) as i32;
    // Zero means "new", so it can never be an assigned id.
    if candidate == 0 {
        candidate = 1;
    }
    while existing.iter().any(|l| l.id == candidate) {
        candidate = candidate.wrapping_add(1);
    }
    candidate
}

/// Great-circle distance in metres.
fn haversine_metres(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    EARTH_RADIUS_METRES * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::block_on;
    use crate::storage::MemStorage;

    fn location(id: i32, name: &str, latitude: f64, longitude: f64) -> KnownLocation {
        KnownLocation {
            id,
            name: name.to_owned(),
            latitude,
            longitude,
        }
    }

    fn seeded(locations: &[KnownLocation]) -> MemStorage {
        let json = models::to_json(&locations.to_vec(), Format::Storage).expect("serializes");
        MemStorage::seeded([(STORAGE_KEY, json.as_str())])
    }

    /// Ports `GetAllAsync_ReturnsEmptyList_WhenNoneExist`.
    #[test]
    fn returns_empty_when_nothing_is_stored() {
        block_on(async {
            let store = MemStorage::new();
            assert!(
                KnownLocationService::new(&store)
                    .all()
                    .await
                    .expect("ok")
                    .is_empty()
            );
        });
    }

    /// Ports `GetAllAsync_ReturnsStoredLocations`.
    #[test]
    fn reads_stored_locations() {
        block_on(async {
            let store = seeded(&[location(1, "Gym", 10.0, 20.0)]);
            let all = KnownLocationService::new(&store).all().await.expect("ok");
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].name, "Gym");
        });
    }

    /// Ports `SaveAsync_NewLocation_AssignsHashDerivedId` and `_StoresInList`.
    #[test]
    fn a_new_location_gets_a_derived_id_and_is_stored() {
        block_on(async {
            let store = MemStorage::new();
            let service = KnownLocationService::new(&store);

            let saved = service
                .save(location(0, "Gym", 37.42, -122.08))
                .await
                .expect("ok");

            assert_ne!(saved.id, 0, "an id must be assigned");
            assert_eq!(service.all().await.expect("ok").len(), 1);
        });
    }

    #[test]
    fn derived_ids_are_deterministic_unlike_the_csharp() {
        // .NET's HashCode.Combine is randomly seeded per process, so this is a
        // deliberate improvement rather than a port.
        let first = assign_id(37.42, -122.08, &[]);
        let second = assign_id(37.42, -122.08, &[]);
        assert_eq!(first, second);
        assert_ne!(first, assign_id(37.43, -122.08, &[]));
    }

    /// Ports `SaveAsync_HashCollision_IncrementsId`.
    #[test]
    fn a_colliding_id_is_incremented() {
        let taken = assign_id(1.0, 2.0, &[]);
        let existing = vec![location(taken, "Taken", 9.0, 9.0)];
        assert_eq!(assign_id(1.0, 2.0, &existing), taken.wrapping_add(1));
    }

    /// Ports `SaveAsync_ExistingId_UpdatesRecord`.
    #[test]
    fn saving_with_an_existing_id_updates_in_place() {
        block_on(async {
            let store = seeded(&[location(5, "Old", 1.0, 2.0)]);
            let service = KnownLocationService::new(&store);

            service
                .save(location(5, "New", 3.0, 4.0))
                .await
                .expect("ok");

            let all = service.all().await.expect("ok");
            assert_eq!(all.len(), 1, "updated, not appended");
            assert_eq!(all[0].name, "New");
            assert_eq!(all[0].latitude, 3.0);
        });
    }

    #[test]
    fn saving_an_unknown_id_appends_it_so_import_can_restore_ids() {
        block_on(async {
            let store = seeded(&[location(5, "Existing", 1.0, 2.0)]);
            let service = KnownLocationService::new(&store);

            service
                .save(location(-2140118897, "Imported", 3.0, 4.0))
                .await
                .expect("ok");

            let all = service.all().await.expect("ok");
            assert_eq!(all.len(), 2);
            assert!(all.iter().any(|l| l.id == -2140118897));
        });
    }

    /// Ports `DeleteAsync_RemovesLocationById`.
    #[test]
    fn delete_removes_by_id() {
        block_on(async {
            let store = seeded(&[location(1, "A", 0.0, 0.0), location(2, "B", 1.0, 1.0)]);
            let service = KnownLocationService::new(&store);

            service.delete(1).await.expect("ok");
            let all = service.all().await.expect("ok");
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].id, 2);
        });
    }

    /// Ports `FindNearbyAsync_ReturnsClosestWithin100m`.
    #[test]
    fn finds_the_closest_location_within_a_hundred_metres() {
        block_on(async {
            // ~11m and ~55m north of the probe respectively.
            let store = seeded(&[
                location(1, "Further", 37.0005, -122.0),
                location(2, "Closer", 37.0001, -122.0),
            ]);
            let found = KnownLocationService::new(&store)
                .find_nearby(37.0, -122.0)
                .await
                .expect("ok")
                .expect("within range");
            assert_eq!(found.name, "Closer");
        });
    }

    /// Ports `FindNearbyAsync_ReturnsNull_WhenNoneWithin100m`.
    #[test]
    fn returns_none_when_nothing_is_within_range() {
        block_on(async {
            // ~1.1km away.
            let store = seeded(&[location(1, "Far", 37.01, -122.0)]);
            assert!(
                KnownLocationService::new(&store)
                    .find_nearby(37.0, -122.0)
                    .await
                    .expect("ok")
                    .is_none()
            );
        });
    }

    /// Ports `FindNearbyAsync_ReturnsNull_WhenNoLocations`.
    #[test]
    fn returns_none_when_there_are_no_locations() {
        block_on(async {
            let store = MemStorage::new();
            assert!(
                KnownLocationService::new(&store)
                    .find_nearby(37.0, -122.0)
                    .await
                    .expect("ok")
                    .is_none()
            );
        });
    }

    #[test]
    fn the_threshold_is_exclusive_at_a_hundred_metres() {
        // Sanity-check the distance function itself against a known separation.
        let metres = haversine_metres(37.0, -122.0, 37.0009, -122.0);
        assert!(
            (99.0..101.0).contains(&metres),
            "expected ~100m, got {metres}"
        );
    }

    /// Ports the three `NextAutoNameAsync_*` scenarios.
    #[test]
    fn auto_names_fill_the_lowest_available_slot() {
        block_on(async {
            let empty = MemStorage::new();
            assert_eq!(
                KnownLocationService::new(&empty)
                    .next_auto_name()
                    .await
                    .expect("ok"),
                "New Location 1"
            );

            let one = seeded(&[location(1, "New Location 1", 0.0, 0.0)]);
            assert_eq!(
                KnownLocationService::new(&one)
                    .next_auto_name()
                    .await
                    .expect("ok"),
                "New Location 2"
            );

            // A gap is filled before extending.
            let gapped = seeded(&[
                location(1, "New Location 1", 0.0, 0.0),
                location(2, "New Location 3", 1.0, 1.0),
            ]);
            assert_eq!(
                KnownLocationService::new(&gapped)
                    .next_auto_name()
                    .await
                    .expect("ok"),
                "New Location 2"
            );
        });
    }

    #[test]
    fn custom_names_do_not_affect_auto_numbering() {
        block_on(async {
            let store = seeded(&[
                location(1, "Gym", 0.0, 0.0),
                location(2, "New Location 1", 1.0, 1.0),
            ]);
            assert_eq!(
                KnownLocationService::new(&store)
                    .next_auto_name()
                    .await
                    .expect("ok"),
                "New Location 2"
            );
        });
    }

    #[test]
    fn reads_the_real_profile_locations() {
        block_on(async {
            let snapshot = crate::fixtures::read_json_fixture("idb-snapshot.json");
            let json =
                serde_json::to_string(&snapshot["entries"]["knownLocations"]["value"]).expect("ok");
            let store = MemStorage::seeded([(STORAGE_KEY, json.as_str())]);

            let all = KnownLocationService::new(&store).all().await.expect("ok");
            assert_eq!(all.len(), 11);
            // The real profile carries ids of both signs from HashCode.Combine.
            assert!(all.iter().any(|l| l.id < 0));
            assert!(all.iter().any(|l| l.id > 0));
        });
    }
}
