<?php
namespace Tests\Feature;
use Tests\TestCase;
use App\Contracts\GalleryProvider;
use App\Support\{ApiFailure, HtmlSanitizer};
final class SpaceContractTest extends TestCase {
    public function test_gallery_can_be_replaced_through_container(): void {
        $this->app->bind(GalleryProvider::class, fn () => new class implements GalleryProvider {
            public function feed(array $filters): array { return ['source'=>'fixture','count'=>1,'items'=>[['url'=>'https://example.org/image.jpg']]]; }
        });
        $this->getJson('/api/jwst/feed')->assertOk()->assertJsonPath('source','fixture')->assertJsonPath('count',1);
    }
    public function test_api_failure_is_http_200_with_trace(): void {
        $this->app->bind(GalleryProvider::class, fn () => new class implements GalleryProvider {
            public function feed(array $filters): array { throw new ApiFailure('UPSTREAM_403', 'External provider rejected request'); }
        });
        $this->getJson('/api/jwst/feed')->assertOk()->assertJsonPath('ok',false)->assertJsonPath('error.code','UPSTREAM_403')->assertJsonStructure(['error'=>['trace_id']]);
    }
    public function test_invalid_inputs_never_call_provider(): void {
        $this->getJson('/api/jwst/feed?perPage=999')->assertOk()->assertJsonPath('error.code','VALIDATION_ERROR');
    }
    public function test_cms_html_sanitizer_preserves_markup_and_removes_xss(): void {
        $sanitizer = new HtmlSanitizer();
        $html = $sanitizer->clean('<h3>Title</h3><script>alert(1)</script><a href="javascript:alert(1)" onclick="evil()">click</a><p>Text</p>');
        $this->assertStringContainsString('<h3>Title</h3>', $html);
        foreach (['<script','javascript:','onclick','alert(1)'] as $bad) $this->assertStringNotContainsString($bad, $html);
    }
}
