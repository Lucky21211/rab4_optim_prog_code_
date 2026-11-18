<?php
namespace App\Services;
use App\Repositories\CmsRepository;
use App\Support\{HtmlSanitizer, ApiFailure};
use App\ViewModels\DashboardViewModel;
final class DashboardService {
    public function __construct(private CachedSpaceService $data, private CmsRepository $cms, private HtmlSanitizer $sanitizer) {}
    public function model(): DashboardViewModel {
        $warning = null;
        try { $iss = $this->data->read('/last'); } catch (ApiFailure $e) { $iss = []; $warning = $e->getMessage(); }
        try { $block = $this->cms->find('dashboard_experiment'); $html = $this->sanitizer->clean($block->content ?? ''); }
        catch (\Throwable $e) { $html = ''; $warning ??= 'Блок CMS временно недоступен'; }
        return new DashboardViewModel($iss, $html, $warning);
    }
}
