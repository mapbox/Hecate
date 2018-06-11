<template>
<div class='flex-parent flex-parent--column viewport-third h-auto-ml hmax-full bg-white round-ml shadow-darken10' style="pointer-events:auto;">
    <template v-if='delta'>
        <div class='flex-child px12 py12'>
            <h3 class='fl py6 txt-m txt-bold'>Delta #<span v-text="delta.id"></span></h3>
            <button @click="delta = false" class='fr btn round bg-gray-light bg-darken25-on-hover color-gray-dark fl'><svg class='icon'><use href='#icon-arrow-left'/></button>
        </div>

        <div class="flex-child py12 px12 bg-gray-faint txt-s align-center">
            <span v-text="delta.props.message ? delta.props.message : '<No Delta Message>'"></span>
        </div>

        <div class="flex-child scroll-auto">
            <div v-for="(feat, feat_it) in delta.features.features" class="px12 py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
                <span v-if="feat.geometry && feat.geometry.type === 'Point'" class="fl py6 px6"><svg class='icon'><use href='#icon-marker'/></span>
                <span v-else-if="feat.geometry && feat.geometry.type === 'MultiPoint'" class="fl px6 py6"><svg class='icon'><use href='#icon-marker'/></span>
                <span v-else-if="feat.geometry && feat.geometry.type === 'LineString'" class="fl px6 py6"><svg class='icon'><use href='#icon-polyline'/></span>
                <span v-else-if="feat.geometry && feat.geometry.type === 'MultiLineString'" class="fl px6 py6"><svg class='icon'><use href='#icon-polyline'/></span>
                <span v-else-if="feat.geometry && feat.geometry.type === 'Polygon'" class="fl px6 py6"><svg class='icon'><use href='#icon-polygon'/></span>
                <span v-else-if="feat.geometry && feat.geometry.type === 'MultiPolygon'" class="fl px6 py6"><svg class='icon'><use href='#icon-polygon'/></span>
                <span v-else class="fl px6 py6"><svg class='icon'><use href='#icon-circle'/></span>

                <span class="fl" v-text="feat.id"></span>
                <span class="fl px6" v-text="feat.action"></span>
            </div>
        </div>
    </template>
    <template v-else>
        <div class='flex-child px12 py12 border--gray-light border-b'>
            <h3 class='fl py6 txt-m txt-bold'>Deltas</h3>
            <button @click="getDeltas" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-refresh'/></button>
        </div>

        <div v-if="!deltas.length" class="px12 py3 clearfix bg-white">
            <div align="center">No Deltas</div>
        </div>

        <div class="flex-child scroll-auto">
            <div v-for="delta in deltas" @click="getDelta(delta.id)" class="px12 py12 border-b bg-darken10-on-hover border--gray-light cursor-pointer wfull">
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
    mounted: function() {
        this.moment = Moment;
    },
    created: function() {
        this.getDeltas();
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
        },
        style: function() {
            let action_create = '#008000';
            let action_modify = '#FFFF00';
            let action_delete = '#FF0000';

            this.map.layers.push('hecate-delta-polygons');
            this.map.gl.addLayer({
                id: 'hecate-delta-polygons',
                type: 'fill',
                source: 'hecate-delta',
                filter: ['==', '$type', 'Polygon'],
                paint: {
                    'fill-opacity': 0.4,
                    'fill-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ]
                }
            });
            this.map.layers.push('hecate-delta-polygon-outlines');
            this.map.gl.addLayer({
                id: 'hecate-delta-polygon-outlines',
                type: 'line',
                source: 'hecate-delta',
                filter: ['==', '$type', 'Polygon'],
                layout: {
                    'line-join': 'round',
                    'line-cap': 'round'
                },
                paint: {
                    'line-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ],
                    'line-width': 0.75
                }
            })
            this.map.layers.push('hecate-delta-lines');
            this.map.gl.addLayer({
                id: 'hecate-delta-lines',
                type: 'line',
                source: 'hecate-delta',
                filter: ['==', '$type', 'LineString'],
                layout: {
                    'line-join': 'round',
                    'line-cap': 'round'
                },
                paint: {
                    'line-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ],
                    'line-width': 1.25
                }
            });
            this.map.layers.push('hecate-delta-points');
            this.map.gl.addLayer({
                id: 'hecate-delta-points',
                type: 'circle',
                source: 'hecate-delta',
                filter: ['==', '$type', 'Point'],
                paint: {
                    'circle-color': [ 'match', [ 'get', '_action' ], 'create', action_create, 'modify', action_modify, 'delete', action_delete, action_create ],
                    'circle-radius': 4
                }
            });
        }
    },
    watch: {
        delta: function() {
            //Reset Normal Map
            if (!this.delta) {
                this.map.unstyle();
                this.map.default();

                this.map.gl.getSource('hecate-delta').setData({ type: 'FeatureCollection', features: [] });
            } else {
                //Deletes don't have a geometry property and as such
                //should not be dislayed or used to calc. bbox
                const noDeletes = {
                    type: 'FeatureCollection',
                    features: this.delta.features.features.filter((feat) => {
                        if (!feat.geometry) return false;

                        return true;
                    })
                };

                this.map.gl.getSource('hecate-delta').setData(noDeletes);

                this.map.unstyle();
                this.style();

                this.delta.bbox = turf.bbox(noDeletes);
                this.map.gl.fitBounds(this.delta.bbox);
            }
        }
    },
    render: h => h(App),
    props: [ 'map' ]
}
</script>
