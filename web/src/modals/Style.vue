<template>
<div class='absolute top left bottom right z3' style="pointer-events: none;">
    <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
        <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
            <div class='grid w-full'>
                <div class='col col--11'>
                    <template v-if="credentials.uid === modal.style_set.uid">
                        <input class='input my12' v-model="modal.style_set.name" placeholder="Style Name"/>
                    </template>
                    <template v-else>
                        <h3 class='fl py6 txt-m txt-bold fl'><span v-text='`${modal.style_set.username}/${modal.style_set.name}`'></span></h3>
                    </template>
                </div>

                <div class='col col--1'>
                    <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                </div>

                <div class='col col--12'>
                    <textarea :readonly="credentials.uid !== modal.style_set.uid" class='textarea w-full h360' v-model="modal.style_set.style" placeholder="Style JSON"></textarea>
                </div>

                <div class='col col--12'>
                    <div class='grid grid--gut12'>
                        <div class='col col--8 py12'>
                            <template v-if="credentials.authed && credentials.uid !== modal.style_set.uid">
                                <button @click='style_create(modal.style_set.style, modal.style_set.name)' class='btn round btn--stroke w-full'>Clone &amp; Edit</button>
                            </template>
                            <template v-else-if="credentials.authed && modal.style_set.id">
                                <label class='switch-container my6'>
                                    <input type='checkbox' v-model="modal.style_set.public"/>
                                    <div class='switch mr6'></div>
                                    Public Style
                                </label>
                            </template>
                        </div>

                        <div class='col col--4 py12'>
                            <template v-if="credentials.uid === modal.style_set.uid">
                                <button @click='style_update(modal.style_set.id, modal.style_set.name, JSON.parse(modal.style_set.style))' class='btn round btn--stroke w-full'>Save &amp; Apply Style</button>
                            </template>
                            <template v-else>
                                <button @click='style_set(modal.style_set.id, JSON.parse(modal.style_set.style))' class='btn round btn--stroke w-full'>Apply Style</button>
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
        return { }
    },
    methods: {
        close: function() {
            this.$emit('close');
        }
    },
    render: h => h(App),
    props: []
}
</script>
