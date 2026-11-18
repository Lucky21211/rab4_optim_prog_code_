<?php
namespace App\Http\Controllers;
use Illuminate\Http\Request;
use App\Contracts\GalleryProvider;
use App\Services\DashboardService;
final class DashboardController extends Controller {
    public function index(DashboardService $service) { return view('dashboard', $service->model()->toArray()); }
    public function jwstFeed(Request $r, GalleryProvider $gallery) {
        $filters = $r->validate([
            'source' => 'sometimes|in:jpg,suffix,program', 'suffix' => 'nullable|regex:/^[A-Za-z0-9_-]{1,32}$/',
            'program' => 'nullable|regex:/^[0-9]{1,16}$/', 'instrument' => 'nullable|in:NIRCam,MIRI,NIRISS,NIRSpec,FGS',
            'page' => 'sometimes|integer|min:1|max:10000', 'perPage' => 'sometimes|integer|min:1|max:60',
        ]);
        return response()->json($gallery->feed(array_filter($filters, fn ($v) => $v !== null && $v !== '')));
    }
}
