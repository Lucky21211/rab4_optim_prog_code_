<?php
namespace App\Support;
final class HtmlSanitizer {
    public function clean(string $html): string {
        $doc = new \DOMDocument('1.0', 'UTF-8');
        $previous = libxml_use_internal_errors(true);
        $doc->loadHTML('<?xml encoding="UTF-8"><div id="safe-root">'.$html.'</div>', LIBXML_NONET | LIBXML_HTML_NOIMPLIED | LIBXML_HTML_NODEFDTD);
        libxml_clear_errors(); libxml_use_internal_errors($previous);
        $allowed = ['div','p','br','h1','h2','h3','h4','h5','h6','ul','ol','li','strong','em','b','i','a','code','pre','span','blockquote'];
        $walk = function (\DOMNode $node) use (&$walk, $allowed) {
            foreach (iterator_to_array($node->childNodes) as $child) {
                if ($child instanceof \DOMElement) {
                    if (!in_array(strtolower($child->tagName), $allowed, true)) { $node->removeChild($child); continue; }
                    foreach (iterator_to_array($child->attributes) as $attr) {
                        $safeHref = $child->tagName === 'a' && $attr->name === 'href' && preg_match('~^https?://~i', $attr->value);
                        if (!$safeHref) $child->removeAttribute($attr->name);
                    }
                }
                if ($child instanceof \DOMComment) { $node->removeChild($child); continue; }
                $walk($child);
            }
        };
        $walk($doc);
        $root = $doc->getElementsByTagName('div')->item(0);
        if (!$root) return '';
        $result = ''; foreach ($root->childNodes as $node) $result .= $doc->saveHTML($node);
        return $result;
    }
}
