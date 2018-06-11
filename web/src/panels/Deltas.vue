<template>
<div v-if="panel === 'Deltas'" class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
    <template v-if='delta'>
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
    </template>
    <template v-else>
        <div class='flex-child px12 py12 bg-gray-faint round-b-ml txt-s'>
            <template><panel v-model="panel"/></template>
            <button @click="deltas_refresh" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-refresh'/></button>
        </div>

        <div v-if="!deltas.length" class="px12 py3 clearfix bg-white">
            <div align="center">No Deltas</div>
        </div>

        <div class="flex-child scroll-auto">
            <div v-for="delta in deltas" @click="delta_get(delta.id)" class="px12 py12 border-b bg-darken10-on-hover border--gray-light cursor-pointer wfull">
                <div class="clearfix">
                    <div class="fl txt-bold" v-text="delta.username"></div>
                    <div class="fr txt-em" v-text="moment(delta.created).add(moment().utcOffset(), 'minutes').fromNow()"></div>
                </div>
                <span v-text="delta.props.message ? delta.props.message : '<No Delta Message>'"></span>
                <span class='bg-blue-faint color-blue inline-block px6 py3 my3 my3 txt-xs txt-bold round fr' v-text="delta.id"></span>
            </div>
        </div>
    </template>

    <foot/>
</div>
</template>

<script>
import Moment from 'moment';

export default {
    name: 'deltas',
    data: function() {
        return {
            deltas: [],
            delta: false
        }
    },
    methods: {
        getDeltas: function() {
            fetch(`http://${window.location.host}/api/deltas`).then((response) => {
                  return response.json();
            }).then((body) => {
                this.deltas.splice(0, this.deltas.length);
                this.deltas = this.deltas.concat(body);
            });
        },
        getDelta: function(delta_id) {
            if (!delta_id) return;

            fetch(`http://${window.location.host}/api/delta/${delta_id}`).then((response) => {
                  return response.json();
            }).then((body) => {
                body.features.features = body.features.features.map(feat => {
                    feat.properties._action = feat.action;
                    return feat;
                });
                this.delta = body;
            });
        }
    },
    watch: {
        delta: function() {
            //Reset Normal Map
            if (!this.delta) {
                this.map_unstyle();
                this.map_default_style();

                this.map.getSource('hecate-delta').setData({ type: 'FeatureCollection', features: [] });
            } else {
                this.map.getSource('hecate-delta').setData(this.delta.features);
                this.map_unstyle();
                this.map_delta_style();

                this.delta.bbox = turf.bbox(this.delta.features);
                this.map.fitBounds(this.delta.bbox);
            }
        }
    },
    created: function() {
        this.moment = Moment;
        this.getDeltas();
    },
    render: h => h(App),
    props: [ 'panel', 'map' ]
}
</script>
