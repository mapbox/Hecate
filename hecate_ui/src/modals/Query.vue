<template>
<div class='absolute top left bottom right z3' style="pointer-events: none;">
    <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
        <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
            <div class='grid w-full'>
                <template v-if='!results.length'>
                    <div class='pb12 col col--11'>
                        <h3 class='fl txt-m txt-bold fl'>SQL Query Editor</h3>
                    </div>

                    <div class='col col--1'>
                        <button @click='close()' class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                    </div>

                    <div class='col col--12'>
                        <textarea :readonly="!credentials.uid" class='textarea w-full h360' v-model="query" placeholder="Query SQL"></textarea>
                    </div>

                    <div class='col col--12'>
                        <p>Note the web UI only supports querying up to 100 features</p>
                    </div>
                    <div class='col col--12'>
                        <div class='grid grid--gut12'>
                            <div class='col col--8 py12'></div>
                            <div class='col col--4 py12'>
                                <button @click="getQuery(query)" class='btn round btn--stroke w-full'>Query</button>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else>
                    <div class='pb12 col col--11'>
                        <button @click="results = ''" class='btn round bg-gray-light bg-darken25-on-hover color-gray-dark fl'><svg class='icon'><use href='#icon-arrow-left'/></button>
                        <h3 class='fl pl12 txt-m txt-bold fl'>SQL Results Viewer</h3>
                    </div>

                    <div class='col col--1'>
                        <button @click='close()' class='fr btn round bg-white color-black bg-darken25-on-hover color-gray-dark fl'><svg class='icon'><use href='#icon-arrow-left'/></button>
                    </div>

                    <div class='col col--12'>
                        <textarea readonly class='textarea w-full h360' v-model="results" placeholder="SQL Results"></textarea>
                    </div>

                    <div class='col col--12'>
                        <div class='grid grid--gut12'>
                            <div class='col col--8 py12'></div>
                            <div class='col col--4 py12'>
                                <button @click="close()" class='btn round btn--stroke w-full'>Close</button>
                            </div>
                        </div>
                    </div>
                </template>
            </div>
        </div>
    </div>
</div>
</template>

<script>
export default {
    name: 'query',
    data: function() {
        return {
            query: '',
            results: []
        }
    },
    methods: {
        close: function() {
            this.$emit('close');
        },
        getQuery: function(query) {
            fetch(`${window.location.protocol}//${window.location.host}/api/data/query?limit=100&query=${encodeURIComponent(this.query.replace(/;/g, ''))}`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                  return response.text();
            }).then((body) => {
                this.results = body;
            });
        }
    },
    render: h => h(App),
    props: ['credentials', 'auth']
}
</script>
