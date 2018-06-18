<template>
<div class='absolute top left bottom right z3' style="pointer-events: none;">
    <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
        <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
            <div class='grid w-full'>
                <div class='col col--11'>
                    <template v-if="credentials.uid === uid">
                        <input class='input mb12' v-model="name" placeholder="Style Name"/>
                    </template>
                    <template v-else>
                        <h3 class='fl py6 txt-m txt-bold fl'><span v-text='`${username}/${name}`'></span></h3>
                    </template>
                </div>

                <div class='col col--1'>
                    <button @click='close()'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                </div>

                <div class='col col--12'>
                    <textarea :readonly="credentials.uid !== uid" class='textarea w-full h360' v-model="style" placeholder="Style JSON"></textarea>
                </div>

                <div class='col col--12'>
                    <div class='grid grid--gut12'>
                        <div class='col col--8 py12'>
                            <template v-if="credentials.authed && credentials.uid !== uid">
                                <button @click='style_create(style, name)' class='btn round btn--stroke w-full'>Clone &amp; Edit</button>
                            </template>
                            <template v-else-if="credentials.authed && id">
                                <label class='switch-container my6'>
                                    <input type='checkbox' v-model="public"/>
                                    <div class='switch mr6'></div>
                                    Public Style
                                </label>
                            </template>
                        </div>

                        <div class='col col--4 py12'>
                            <template v-if="credentials.uid === uid">
                                <button @click='style_update(id, name, JSON.parse(style))' class='btn round btn--stroke w-full'>Save &amp; Apply Style</button>
                            </template>
                            <template v-else>
                                <button @click='setStyle(id, JSON.parse(style))' class='btn round btn--stroke w-full'>Apply Style</button>
                            </template>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
</template>

<script>
export default {
    name: 'style',
    data: function() {
        return {
            id: false,
            public: false,
            uid: false,
            username: false,
            name: '',
            style: '',


        }
    },
    created: function() {
        this.getStyle(this.id);
    },
    watch: {
        style: function() {
            this.getStyle(this.id);
        }
    },
    methods: {
        close: function() {
            this.$emit('close');
        },
        getStyle: function(style_id) {
            if (!style_id) return;
            fetch(`http://${window.location.host}/api/style/${style_id}`, {
                credentials: 'same-origin'
            }).then((response) => {
                  return response.json();
            }).then((body) => {
                this.style = JSON.stringify(body.style, null, 4);
                this.id = body.id;
                this.name = body.name;
                this.public = body.public;
                this.uid = body.uid;
                this.username = body.username;
            })
        },
        createStyle: function(style, name) {
            this.style = style ? style : '';
            this.id = false;
            this.username = this.credentials.username;
            this.uid = this.credentials.uid;
            this.name = name ? `Copy of ${name}` : '';
            this.public = false;
        },
        updateStyle: function(style_id, style_name, style) {
            if (!style_id) { //Create new style
                fetch(`http://${window.location.host}/api/style`, {
                    method: 'POST',
                    credentials: 'same-origin',
                    headers: new Headers({
                        'Content-Type': 'application/json'
                    }),
                    body: JSON.stringify({
                        name: style_name,
                        style: style
                    })
                }).then((response) => {
                    if (response.status === 200) {
                        this.setStyle(style_id, style);
                    } else {
                        return this.ok('Failed to push style', 'Failed to update style');
                    }
                }).catch((err) => {
                    return this.ok('Failed to push style', 'Failed to update style');
                });
            } else { //Update Existing Style
                fetch(`http://${window.location.host}/api/style/${style_id}`, {
                    method: 'PATCH',
                    credentials: 'same-origin',
                    headers: new Headers({
                        'Content-Type': 'application/json'
                    }),
                    body: JSON.stringify({
                        name: style_name,
                        style: style
                    })
                }).then((response) => {
                    if (response.status !== 200) return this.ok('Failed to push style', 'Failed to update style');
                    if (this.credentials.authed && this.id) {
                        fetch(`http://${window.location.host}/api/style/${style_id}/${this.public ? 'public' : 'private'}`, {
                            method: 'POST',
                            credentials: 'same-origin'
                        }).then((response) => {
                            if (response.status !== 200) return this.ok('Failed to push style', 'Failed to update style');
                            this.setStyle(style_id, style);
                        }).catch((err) => {
                            return this.ok('Failed to push style', 'Failed to update style');
                        });
                    } else {
                        this.setStyle(style_id, style);
                    }
                }).catch((err) => {
                    return this.ok('Failed to push style', 'Failed to update style');
                });
            
            }
        },
        setStyle: function(style_id, style) {
            if (!style.version || style.version !== 8) return this.ok('Style Not Applied', 'The selected style could not be applied. The style version must be 8');
            if (!style.layers || style.layers.length === 0) return this.ok('Style Not Applied', 'The selected style could not be applied. The style must contain at least 1 layer');
            this.map.unstyle();
            for (let layer of style.layers) {
                if (!layer.id) {
                    this.map.unstyle();
                    this.map.default();
                    return this.ok('Style Not Applied', 'Every layer in the style must have a unique id');
                }
                layer.source = 'hecate-data';
                layer['source-layer'] = 'data',
                this.map.layers.push(layer.id);
                this.map.gl.addLayer(layer);
            }
            this.$emit('close');
        },
        setStyleModal: function(style_id) {
            this.getStyle(style_id, (err, style) => {
                if (err) return this.ok('Failed to retrieve style', err.message);
                this.style = JSON.stringify(style.style, null, 4);
                this.id = style.id;
                this.username = style.username;
                this.uid = style.uid;
                this.name = style.name;
                this.public = style.public;
            });
        }
    },
    render: h => h(App),
    props: ['id', 'map', 'credentials']
}
</script>
