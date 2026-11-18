<?php
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\{Schema, DB};
return new class extends Migration {
    public function up(): void {
        if (!Schema::hasTable('cms_blocks')) Schema::create('cms_blocks', function (Blueprint $table) {
            $table->id(); $table->string('slug')->unique(); $table->text('title'); $table->text('content'); $table->boolean('is_active')->default(true);
        });
        if (Schema::hasTable('cms_pages')) foreach (DB::table('cms_pages')->cursor() as $page) {
            DB::table('cms_blocks')->insertOrIgnore(['slug' => $page->slug, 'title' => $page->title, 'content' => $page->body, 'is_active' => true]);
        }
        DB::table('cms_blocks')->insertOrIgnore(['slug' => 'dashboard_experiment', 'title' => 'Кассиопея', 'content' => '<p>Космические данные и телеметрия</p>', 'is_active' => true]);
    }
    public function down(): void { /* Preserve existing CMS content on rollback. */ }
};
