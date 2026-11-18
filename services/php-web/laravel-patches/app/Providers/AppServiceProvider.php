<?php
namespace App\Providers;
use Illuminate\Support\ServiceProvider;
use App\Contracts\{SpaceDataSource, GalleryProvider, EventProvider};
use App\Services\{JwstGalleryProvider, AstronomyEventProvider};
final class AppServiceProvider extends ServiceProvider {
    public function register(): void {
        $this->app->bind(SpaceDataSource::class, \App\Clients\RustClient::class);
        $this->app->bind(GalleryProvider::class, JwstGalleryProvider::class);
        $this->app->bind(EventProvider::class, AstronomyEventProvider::class);
    }
    public function boot(): void {}
}
