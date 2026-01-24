using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Trainer;
using Trainer.Services;

var builder = WebAssemblyHostBuilder.CreateDefault(args);
builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

builder.Services.AddScoped(sp => new HttpClient { BaseAddress = new Uri(builder.HostEnvironment.BaseAddress) });

// Register services
builder.Services.AddScoped<IndexedDbStorageService>();
builder.Services.AddScoped<IStorageService>(sp => sp.GetRequiredService<IndexedDbStorageService>());
builder.Services.AddScoped<IActivityService, ActivityService>();
builder.Services.AddScoped<IActivityTypeService, ActivityTypeService>();
builder.Services.AddScoped<IGoalService, GoalService>();
builder.Services.AddScoped<IExportImportService, ExportImportService>();

await builder.Build().RunAsync();
