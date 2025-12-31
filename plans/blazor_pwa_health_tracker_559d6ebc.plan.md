---
name: Blazor PWA Health Tracker
overview: Create a .NET 8 Blazor WebAssembly Progressive Web App for tracking health activities (water, snacks, activities) with offline support, export/import functionality, and comprehensive unit testing.
todos:
  - id: setup-project
    content: Initialize .NET 8 solution with Blazor WebAssembly project and xUnit test project
    status: completed
  - id: configure-pwa
    content: Configure PWA features (manifest.json, service worker, Program.cs)
    status: completed
  - id: implement-models
    content: Create Activity and ActivityType data models
    status: completed
  - id: implement-storage
    content: Implement LocalStorageService for browser storage
    status: completed
  - id: implement-services
    content: Implement ActivityService, ActivityTypeService, and ExportImportService
    status: completed
  - id: create-main-ui
    content: Create main Index page with chart, activity grid, and action buttons
    status: completed
  - id: create-activity-form
    content: Create ActivityEntry component for adding/editing activities
    status: completed
  - id: create-activity-type-form
    content: Create ActivityTypeEntry component for managing activity types
    status: completed
  - id: add-charting
    content: Integrate Chart.js for activity type graph visualization
    status: completed
  - id: implement-tests
    content: Write unit tests for all services to achieve >90% coverage
    status: completed
  - id: setup-github-actions
    content: Create GitHub Actions workflows for testing and deployment
    status: completed
  - id: style-ui
    content: Apply styling to match UI mockups and ensure responsive design
    status: completed
---

# Blazor PWA Health Tracker Implementation Plan

## Project Structure

Create a solution with the following projects:

- `Trainer` - Main Blazor WebAssembly PWA application
- `Trainer.Tests` - Unit test project (xUnit)

## Core Components

### 1. Data Models (`Trainer/Models/`)

- `Activity.cs` - Represents a health activity entry
- Properties: Id, ActivityTypeId, When (DateTime), Amount (int), Notes (string)
- `ActivityType.cs` - Represents an activity type definition
- Properties: Id, Name (string), NetBenefit (enum: Positive, Negative, None), DailyAmount (int?), MonthlyAmount (int?)

### 2. Services (`Trainer/Services/`)

- `IStorageService.cs` / `LocalStorageService.cs` - Handles browser localStorage for data persistence
- `IActivityService.cs` / `ActivityService.cs` - Manages activity CRUD operations
- `IActivityTypeService.cs` / `ActivityTypeService.cs` - Manages activity type CRUD operations
- `IExportImportService.cs` / `ExportImportService.cs` - Handles JSON export/import functionality

### 3. UI Components (`Trainer/Components/` or `Trainer/Pages/`)

- `Index.razor` - Main screen with:
- Chart component (using Chart.js or similar via JSInterop)
- Activity grid (Blazor table with clickable rows)
- Import/Export/Add buttons
- `ActivityEntry.razor` - Add/Edit activity form
- Activity type dropdown with "+" button
- DateTime picker (defaults to now)
- Amount input (number, whole numbers only)
- Notes textarea
- `ActivityTypeEntry.razor` - Add/Edit activity type form
- Name input
- Net Benefit toggle buttons (green positive, red negative, neither)
- Daily/Monthly amount inputs (whole numbers)

### 4. PWA Configuration

- `manifest.json` - PWA manifest for installability
- `service-worker.js` - Service worker for offline functionality
- Configure in `Program.cs` to enable PWA features

### 5. Styling

- Use Bootstrap or Tailwind CSS for modern UI
- Match the UI mockups from the provided images

## Testing Strategy

### Unit Tests (`Trainer.Tests/`)

- Test all service classes (>90% coverage)
- Test data serialization/deserialization
- Test export/import functionality
- Mock localStorage for testing

## GitHub Actions

### 1. Test Workflow (`.github/workflows/test.yml`)

- Runs on pull requests
- Restores .NET dependencies
- Runs unit tests
- Reports test results

### 2. Deploy Workflow (`.github/workflows/deploy.yml`)

- Runs on release tags
- Builds Blazor WebAssembly app
- Publishes to GitHub Pages

## Implementation Steps

1. Initialize .NET solution and projects
2. Configure Blazor WebAssembly with PWA support
3. Implement data models
4. Implement storage service (localStorage)
5. Implement business services (Activity, ActivityType, ExportImport)
6. Create UI components (Main, ActivityEntry, ActivityTypeEntry)
7. Add charting functionality for activity type graph
8. Implement unit tests for all services
9. Configure PWA manifest and service worker
10. Set up GitHub Actions workflows
11. Add styling to match UI mockups

## Technical Decisions

- **Storage**: Browser localStorage (simple, works offline, easy to test)
- **Charting**: Chart.js via JSInterop (lightweight, works offline)
- **Testing**: xUnit with Moq for mocking