<template>
<div class='flex-parent flex-parent--column viewport-third h-auto-ml bg-white round-ml shadow-darken10' style="pointer-events:auto; max-height: calc(100% - 80px);">
    <div class='flex-child px12 py12'>
        <h3 class='fl py6 txt-m txt-bold'>Styles</h3>
        <button @click="getStyles()" class='btn round bg-gray-light bg-darken10-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-refresh'/></button>

        <template v-if="credentials.authed">
            <button @click="styleModal(false)" class='fr btn mx6 btn--s round align-center'>New</button>
        </template>
    </div>

    <div class="flex-child scroll-auto">
        <div v-if="!styles.length" class="px12 py3 clearfix bg-white">
            <div align="center">No Custom Styles</div>
        </div>

        <div v-for="(style, style_it) in pstyles" class="w-full py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
            <div @click="styleModal(style.id)" class="w-full clearfix">
                <span class="fl py6 px6"><svg class='icon'><use href='#icon-lock'/></span>
                <div class="fl">
                    <span v-text="style.username"></span>/<span v-text="style.name"></span>
                </div>
            </div>
        </div>

        <div v-for="(style, style_it) in styles" class="w-full py3 clearfix bg-white bg-darken25-on-hover cursor-pointer">
            <div @click="styleModal(style.id)" class="w-full clearfix">
                <span class="fl py6 px6"><svg class='icon'><use href='#icon-paint'/></span>
                <div class="fl">
                    <span v-text="style.username"></span>/<span v-text="style.name"></span>
                </div>
            </div>
        </div>
    </div>

    <foot/>
</div>
</template>

<script>
import Foot from '../components/Foot.vue';

export default {
    name: 'styles',
    data: function() {
        return {
            styles: [],
            pstyles: []
        }
    },
    created: function() {
        this.getStyles();
    },
    components: {
        foot: Foot
    },
    methods: {
        styleModal: function(id) {
            this.$emit('style', id);
        },
        getStyles: function() {
            fetch(`${window.location.protocol}//${window.location.host}/api/styles`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                  return response.json();
            }).then((body) => {
                this.styles = body;
            });

            if (this.credentials.authed) {
                fetch(`${window.location.protocol}//${window.location.host}/api/styles/${this.credentials.uid}`, {
                    method: 'GET',
                    credentials: 'same-origin'
                }).then((response) => {
                      return response.json();
                }).then((body) => {
                    this.pstyles = body.filter((style) => {
                        if (style.public) return false;
                        return true;
                    });
                });
            } else {
                this.pstyles = [];
            }
        }
    },
    render: h => h(App),
    props: ['credentials']
}
</script>
