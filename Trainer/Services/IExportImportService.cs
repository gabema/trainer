namespace Trainer.Services;

internal interface IExportImportService
{
    Task<string> ExportDataAsync();
    Task ImportDataAsync(string jsonData);
}

