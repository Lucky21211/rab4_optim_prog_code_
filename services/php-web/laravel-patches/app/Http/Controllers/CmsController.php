<?php
namespace App\Http\Controllers;
use App\Repositories\CmsRepository;
use App\Support\HtmlSanitizer;
final class CmsController extends Controller {
    public function page(string $slug, CmsRepository $cms, HtmlSanitizer $sanitizer) {
        abort_unless(preg_match('/^[A-Za-z0-9_-]{1,100}$/', $slug), 404);
        $row = $cms->find($slug); abort_unless($row, 404);
        return view('cms.page', ['title' => $row->title, 'html' => $sanitizer->clean($row->content)]);
    }
}
