<template>
<div class='absolute top left bottom right z3' style="pointer-events: none;">
    <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events: auto;">
        <template v-if='error'>
            <div class="flex-child px12 py12 w600 h80 bg-white round-ml shadow-darken10">
                <div class='grid w-full'>
                    <div class='col col--8'>
                        <h3 class='fl py6 txt-m txt-bold w-full'>Register Error!</h3>
                        <p class='color-red' v-text='error'></p>
                    </div>

                    <div class='col col--4'>
                        <button @click='error = ""' class='mt24 btn round w-full'>OK</button>
                    </div>
                </div>
            </div>
        </template>
        <template v-else-if='ok'>
            <div class="flex-child px12 py12 w600 h80 bg-white round-ml shadow-darken10">
                <div class='grid w-full'>
                    <div class='col col--8'>
                        <h3 class='fl py6 txt-m txt-bold w-full'>Account Created!</h3>
                        <p>Your account has been created! You can login now.</p>
                    </div>

                    <div class='col col--4'>
                        <button @click='close()' class='mt24 btn round w-full'>OK</button>
                    </div>
                </div>
            </div>
        </template>
        <template v-else>
            <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
                <div class='grid w-full'>
                    <div class='col col--8'>
                        <h3 class='fl py6 txt-m txt-bold w-full'>Hecate Register</h3>
                    </div>
                    <div class='col col--4'>
                        <button @click='close()'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                    </div>

                    <div class='col col--12 py12'>
                        <label>
                            Username:
                            <input v-model="username" class='input py12' placeholder='username'/>
                        </label>
                    </div>

                    <div class='col col--12 py12'>
                        <label>
                            Email:
                            <input v-model='email' class='input py12' placeholder='email'/>
                        </label>
                    </div>

                    <div class='col col--12 py12'>
                        <label>
                            Password:
                            <input v-model='password' type='password' class='input py12' placeholder='password'/>
                        </label>
                    </div>

                    <div class='col col--12'>
                        <div class='grid grid--gut12'>
                            <div class='col col--6 py12'>
                                <button @click='close()' class='btn round bg-gray w-full'>Cancel</button>
                            </div>

                            <div class='col col--6 py12'>
                                <button @click='register' class='btn round w-full'>Register</button>
                            </div>
                        </div>
                    </div>
                </div>
        </template>
    </div>
</template>

<script>
export default {
    name: 'register',
    data: function() {
        return {
            ok: false,
            error: '',
            username: '',
            password: '',
            email: ''
        }
    },
    methods: {
        close: function() {
            this.$emit('close');
        },
        register: function() {
            const self = this;

            fetch(`http://${window.location.host}/api/user/create?username=${this.username}&password=${this.password}&email=${this.email}`).then((response) => {
                if (response.status === 200) {
                    self.error = '';
                    self.ok = true;
                } else {
                    self.error = 'Failed to Register User Account'
                }
            }).catch((err) => {
                self.error = 'Failed to Register User Account'
            });
        }
    },
    render: h => h(App),
    props: []
}
</script>
