<?php
namespace App\ViewModels;
final readonly class DashboardViewModel {
    public function __construct(public array $iss, public string $cmsHtml, public ?string $warning = null) {}
    public function toArray(): array { return ['iss' => $this->iss, 'cmsHtml' => $this->cmsHtml, 'warning' => $this->warning]; }
}
