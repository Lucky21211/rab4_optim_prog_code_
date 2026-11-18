<?php
namespace App\Http\Controllers;
use Illuminate\Http\Request;
final class UploadController extends Controller {
    public function store(Request $request) {
        $request->validate(['file' => 'required|file|max:2048|mimes:csv,txt']);
        $request->file('file')->store('telemetry-uploads', 'local');
        return back()->with('status', 'Файл загружен');
    }
}
