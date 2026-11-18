<?php
use Illuminate\Database\Migrations\Migration;
use Illuminate\Support\Facades\DB;
return new class extends Migration {
    public function up(): void {
        DB::statement('DELETE FROM telemetry_legacy a USING telemetry_legacy b WHERE a.source_file=b.source_file AND a.recorded_at=b.recorded_at AND a.id<b.id');
        DB::statement('CREATE UNIQUE INDEX IF NOT EXISTS ux_legacy_source_time ON telemetry_legacy(source_file,recorded_at)');
    }
    public function down(): void {}
};
