## ADDED Requirements

### Requirement: Installed PWAs take the update without user action
An installed instance running the previous service worker SHALL migrate to the new build without the user uninstalling, clearing site data, or manually refreshing more than once. The new service worker SHALL claim existing clients rather than waiting for every window to close.

#### Scenario: Installed app updates on next launch
- **WHEN** a user with the app installed under the previous implementation opens it while online
- **THEN** the new build is activated and rendered, no later than the second launch

#### Scenario: Open windows are claimed by the new worker
- **WHEN** the new service worker activates while a window is open
- **THEN** that window is controlled by the new worker without needing to be closed and reopened

### Requirement: Stale caches are purged on activation
Activation SHALL delete every cache not belonging to the current cache version, removing the previous implementation's cached shell and framework assets. Cached assets from the previous implementation SHALL NOT be served after activation.

#### Scenario: Previous cache version is deleted
- **WHEN** the new service worker activates on a profile holding the previous cache
- **THEN** the previous cache is deleted and its entries are no longer served

#### Scenario: No half-updated shell is served
- **WHEN** a user loads the app during or after the update
- **THEN** the served document and the assets it references come from the same build, never a new document paired with previously cached assets

### Requirement: Offline capability survives content-hashed filenames
The service worker SHALL NOT hardcode build-generated asset filenames, because the Rust build emits content-hashed names that change every release. Install-time precaching SHALL cover only stable paths, and hashed assets SHALL be cached at runtime as they are fetched. A missing or renamed asset SHALL NOT cause installation to fail.

#### Scenario: Install succeeds without knowing hashed filenames
- **WHEN** the service worker installs against a build whose asset filenames it has never seen
- **THEN** installation completes successfully

#### Scenario: App works offline after a first online visit
- **WHEN** a user opens the app online, then goes offline and reopens it
- **THEN** the app loads and is fully usable, including the activity chart

#### Scenario: A single unreachable asset does not disable offline mode
- **WHEN** one precache URL cannot be fetched during installation
- **THEN** installation still completes and the remaining assets are cached

### Requirement: Local data survives the cutover
The service worker migration SHALL NOT clear, reset, or otherwise disturb IndexedDB or localStorage. Cache eviction SHALL be scoped to the Cache Storage API only.

#### Scenario: Activity history is intact after updating
- **WHEN** an installed user updates from the previous implementation to the new build
- **THEN** their complete activity history, activity types, known locations, and any in-progress activity are present and unchanged

### Requirement: Deep links continue to resolve on GitHub Pages
Direct navigation to a client-side route SHALL render the corresponding view rather than a hosting 404, both online and from cache. Notification clicks that navigate to an activity route SHALL continue to work.

#### Scenario: Direct navigation to a nested route
- **WHEN** a user navigates directly to the activity detail route for an existing activity
- **THEN** that activity's view renders

#### Scenario: Deep link resolves offline
- **WHEN** a user opens a nested route while offline after a prior online visit
- **THEN** the app shell is served from cache and the route renders

#### Scenario: Notification click opens the activity
- **WHEN** a user clicks an active-activity notification carrying an activity id
- **THEN** an existing window is focused and navigated to that activity's route, or a new window is opened there
