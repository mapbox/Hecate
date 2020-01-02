<template>
    <div class='absolute top left bottom right z5' style="pointer-events: none;">
        <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events: auto;">
            <div class="flex-child px12 py12 w600 h80 bg-white round-ml shadow-darken10">
                <div class='grid w-full col'>
                    <div class='col--12'>
                        <h3 v-if='!title' class='w-full py6 txt-m txt-bold align-center'>Uh Oh!</h3>
                        <h3 v-else class='w-full py6 txt-m txt-bold align-center' v-text='title'></h3>
                    </div>

                    <div class='col--12 py12' v-text='body'></div>

                    <div class='col--12 py12'>
                        <div class='grid grid--gut12'>
                            <div class='col col--6'></div>
                            <div class='col col--6'>
                                <button class='btn round w-full' @click="close">Ok</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        <div>
    </div>
</template>

<script>
export default {
    name: 'err',
    render: h => h(App),
    methods: {
        close: function() {
            this.$emit('close');
        }
    },
    props: ['title', 'body']
}
</script>
