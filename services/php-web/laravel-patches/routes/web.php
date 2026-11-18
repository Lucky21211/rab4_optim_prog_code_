<?php
use Illuminate\Support\Facades\Route;
use App\Http\Controllers\{DashboardController, OsdrController, IssController, ProxyController, AstroController, CmsController, UploadController};
Route::get('/', fn () => redirect('/dashboard'));
Route::get('/dashboard', [DashboardController::class, 'index']);
Route::get('/iss', [IssController::class, 'index']);
Route::get('/osdr', [OsdrController::class, 'index']);
Route::get('/page/{slug}', [CmsController::class, 'page']);
Route::post('/upload', [UploadController::class, 'store']);
Route::middleware(\App\Http\Middleware\ApiEnvelope::class)->group(function () {
    Route::get('/api/iss/last', [ProxyController::class, 'last']);
    Route::get('/api/iss/trend', [ProxyController::class, 'trend']);
    Route::get('/api/jwst/feed', [DashboardController::class, 'jwstFeed']);
    Route::get('/api/astro/events', [AstroController::class, 'events']);
});
