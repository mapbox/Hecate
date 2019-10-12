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
                    <div class='grid grid--gut12 col col--12'>
                        <div class='col col--12 txt-m txt-bold'>
                            Account Settings
                            <button @click='close' class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                        </div>

                        <div class='py6 col col--12 border--gray-light border-b'>
                            <span class='txt-m'>Personal Info</span>
                        </div>

                        <div class='col col--12 py12'>
                            <label>Username</label>
                            <input class='input mb6' v-model='user.username' placeholder='Username' />

                            <label>Email</label>
                            <input class='input mb6' v-model='user.email' placeholder='Email' />
                        </div>

                        <div class='py6 col col--12 border--gray-light border-b'>
                            <span class='txt-m'>Account Security</span>
                        </div>

                        <div class='col col--12 py12'>
                            <label>Current Password</label>
                            <input type=password v-model='pw.current' class='input mb6' placeholder='Current Password' />
                        </div>
                        <div class='col col--5'>
                            <label>New Password</label>
                            <input type=password v-model='pw.newPass' class='input mb6' placeholder='New Password' />
                        </div>
                        <div class='col col--5'>
                            <label>Confirm New Password</label>
                            <input type=password v-model='pw.newConf' class='input mb6' placeholder='New Password' />
                        </div>
                        <div class='col col--2'>
                            <button @click='setPassword' style='margin-top: 22px;' class='btn'>Update</button>
                        </div>

                        <div class='py6 col col--12 border--gray-light border-b'>
                            <span class='txt-m'>JOSM URL</span>
                        </div>

                        <template v-if='url || !auth || auth && auth.default === "public"'>
                            <pre class='pre my6 w-full'><code class='w-full' v-text='url || defaultJOSM()'></code></pre>
                        </template>
                        <template v-else>
                            <div class='col col--12 py12'>
                                Generate an authenticated URL
                                <button @click='createUrl' class='fr btn round btn--stroke'>Generate</button>
                            </div>
                        </template>
                    </div>
                </template>
            </div>
        </div>
    </div>
</template>

<script>
export default {
    name: 'self',
    data: function() {
        return {
            url: '',
            user: false,
            error: false,
            pw: {
                current: '',
                newPass: '',
                newConf: ''
            }
        };
    },
    mounted: function() {
        this.getSelf();
    },
    methods: {
        close: function() {
            this.$emit('close');
        },
        defaultJOSM: function() {
            return window.location.host + '/api';
        },
        createUrl: function() {
            fetch(`${window.location.protocol}//${window.location.host}/api/user/token`, {
                method: 'POST',
                credentials: 'same-origin',
                headers: new Headers({
                    'Content-Type': 'application/json'
                }),
                body: JSON.stringify({
                    name: 'JOSM Token',
                    hours: 336
                })
            }).then((response) => {
                if (response.status !== 200) {
                    return this.error = response.status + ':' + response.statusText;
                }

                return response.json();
            }).then((user) => {
                this.url = `${window.location.origin}/token/${user.token}/api`;
            }).catch((err) => {
                this.error = err.message;
            });

        },
        setPassword: function() {
            if (!this.pw.newPass || !this.pw.newConf || !this.pw.current) {
                this.error = 'No Password Fields Can Be Blank';
                return;
            } else if (this.pw.newPass !== this.pw.newConf) {
                this.pw.newPass = '';
                this.pw.newConf = '';
                this.error = 'New Passwords Must Match!';
                return;
            }

            fetch(`${window.location.protocol}//${window.location.host}/api/user/reset`, {
                method: 'POST',
                credentials: 'same-origin',
                headers: new Headers({
                    'Content-Type': 'application/json'
                }),
                body: JSON.stringify({
                    current: this.pw.current,
                    update: this.pw.newPass
                })
            }).then((response) => {
                if (response.status !== 200) {
                    return this.error = response.status + ':' + response.statusText;
                }
            }).then(() => {
                this.error = 'Password Changed!';
            }).catch((err) => {
                this.error = err.message;
            });

        },
        getSelf: function() {
            fetch(`${window.location.protocol}//${window.location.host}/api/user/info`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                if (response.status !== 200) {
                    return this.error = response.status + ':' + response.statusText;
                }

                return response.json();
            }).then((user) => {
                this.user = user;
            }).catch((err) => {
                this.error = err.message;
            });
        },
    },
    render: h => h(App),
    props: ['auth']
}
</script>
