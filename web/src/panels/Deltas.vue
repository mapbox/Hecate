<template>
<div v-if="panel === 'Deltas'" class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
    <div class='flex-child px12 py12'>
        <h3 class='fl py6 txt-m txt-bold'>Hecate Deltas</h3>
    </div>

    <div class='flex-child clearfix px12 py12 bg-gray-faint round-b-ml txt-s'>
        <button @click="delta = false" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fl'><svg class='icon'><use href='#icon-arrow-left'/></button>
        <div class="px12 py3 txt-bold fl">Delta <span v-text="delta.id"></span></div>
    </div>
    <div class="flex-child py12 px12 bg-gray-faint txt-s align-center"><span v-text="delta.props.message ? delta.props.message : '<No Delta Message>'"></span></div>

    <div class="flex-child scroll-auto">
        <div v-for="(feat, feat_it) in delta.features.features" class="px12 py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
            <span v-if="feat.geometry.type === 'Point'" class="fl py6 px6"><svg class='icon'><use href='#icon-marker'/></span>
            <span v-if="feat.geometry.type === 'MultiPoint'" class="fl px6 py6"><svg class='icon'><use href='#icon-marker'/></span>
            <span v-if="feat.geometry.type === 'LineString'" class="fl px6 py6"><svg class='icon'><use href='#icon-polyline'/></span>
            <span v-if="feat.geometry.type === 'MultiLineString'" class="fl px6 py6"><svg class='icon'><use href='#icon-polyline'/></span>
            <span v-if="feat.geometry.type === 'Polygon'" class="fl px6 py6"><svg class='icon'><use href='#icon-polygon'/></span>
            <span v-if="feat.geometry.type === 'MultiPolygon'" class="fl px6 py6"><svg class='icon'><use href='#icon-polygon'/></span>

            <span class="fl" v-text="feat.id"></span>
            <span class="fl px6" v-text="feat.action"></span>
        </div>
    </div>

    <foot/>
</div>
</template>

<script>
export default {
    name: 'deltas',
    render: h => h(App),
    props: [ 'panel', 'delta' ]
}
</script>
