namespace Trainer.Services;

public interface IExportImportService
{
    Task<string> ExportDataAsync();
    Task ImportDataAsync(string jsonData);
}

