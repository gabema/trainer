## REMOVED Requirements

### Requirement: Guided activity notification
The system previously allowed a user to step through an activity's notes line-by-line via push notifications with Previous/Next actions, storing navigation state in IndexedDB.

**Reason**: The feature is not useful and adds disproportionate complexity (notification permission prompts, IndexedDB-backed state in the service worker, multi-step UI in the activity card overlay).

**Migration**: No migration needed. The notes field on an activity is unaffected; only the guided notification flow is removed.
