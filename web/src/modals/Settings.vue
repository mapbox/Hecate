<template>
<div class='absolute top left bottom right z3' style="pointer-events: none;">
    <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events: auto;">
        <div class="flex-child px12 py12 w600 h80 bg-white round-ml shadow-darken10">
            <div class='grid w-full'>
                <div class='col col--12'>
                    <h3 class='fl py6 txt-m txt-bold w-full'>Settings</h3>

                    <h4 class='fl py6 txt-m w-full border--gray border-b'>Base Layers</h4>

                    <h4 class='fl py6 txt-m w-full border--gray border-b'>Default Style</h4>

                </div>

                <div class='col col--12'>
                    <button @click='close()' class='fr mt24 btn round w-full'>OK</button>
                </div>
            </div>
        </div>
    </div>
</div>
</template>

<script>
export default {
    name: 'settings',
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
