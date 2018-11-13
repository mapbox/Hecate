<template>
    <div class='absolute top left bottom right z3' style="pointer-events: none;">
        <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events: auto;">
            <div class="flex-child px12 py12 w600 h80 bg-white round-ml shadow-darken10">
                <template v-if='error'>
                    <div class='grid w-full col'>
                        <div class='col--12'>
                            <h3 class='w-full py6 txt-m txt-bold align-center'>ERROR!</h3>
                        </div>

                        <div class='col--12 py12' v-text='error'></div>

                        <div class='col--12 py12'>
                            <div class='grid grid--gut12'>
                                <div class='col col--6'></div>
                                <div class='col col--6'>
                                    <button @click='error = false' class='btn round w-full'>Ok</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else>
                    <div class='col col--12'>
                        <div class='col col--12 txt-m txt-bold'>
                            <span v-text='username'></span>User Profile
                            <button @click='close' class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                        </div>

                        <div class='py6 col col--12 border--gray-light border-b'>
                            <span class='txt-m'>Base Layers</span>
                        </div>

                        <div class='col col--12 hmin120 hmax180 clearfix'>
                        </div>
                    </div>
                </template>
            </div>
        </div>
    </div>
</template>

<script>
export default {
    name: 'user',
    data: function() {
        return {
            username: '',
            error: false
        };
    },
    mounted: function() {

    },
    methods: {
        close: function() {
            this.$emit('close');
        }
    },
    render: h => h(App),
    props: ['auth', 'user']
}
</script>
