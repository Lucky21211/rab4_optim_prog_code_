<?php
namespace App\Http\Controllers;
use App\Services\CachedSpaceService;
use App\Support\ApiFailure;
final class IssController extends Controller {
    public function index(CachedSpaceService $data) {
        try { $last = $data->read('/last'); $trend = $data->read('/iss/trend'); }
        catch (ApiFailure $e) { $last = []; $trend = []; }
        return view('iss', compact('last', 'trend') + ['base' => config('space.rust_url')]);
    }
}
