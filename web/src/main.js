import Vue from 'vue';
import App from './App.vue';

window.onload = () => {
    new Vue({
        el: '#app',
        render: h => h(App)
    });
}

window.hecate = {
    tiles: {
        clear: function(cb) {
            fetch(`${window.location.protocol}//${window.location.host}/api/tiles`, {
                method: 'DELETE',
                credentials: 'same-origin'
            }).then((response) => {
                if (response.status !== 200) return cb(new Error(response.status + ':' + response.statusText));
                return cb();
            }).catch((err) => {
                return cb(err);
            });
        }
    },
    meta: {
        get: function(key, cb) {
            if (!key) return cb(new Error('key required'));

            fetch(`${window.location.protocol}//${window.location.host}/api/meta/${key}`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                if (response.status !== 200) return cb(new Error(response.status + ':' + response.statusText));
                return response.json();
            }).then((response) => {
                return cb(response);
            }).catch((err) => {
                return cb(err);
            });
        }
    },
    users: {
        list: function(filer, cb) {
            fetch(`${window.location.protocol}//${window.location.host}/api/users?filter=${filter}`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                if (response.status !== 200) return cb(new Error(response.status + ':' + response.statusText));
                return response.json();
            }).then((users) => {
                return cb(null, users);
            }).catch((err) => {
                return cb(err);
            });
        }
    },
    webhooks: {
        list: function(cb) {
            fetch(`${window.location.protocol}//${window.location.host}/api/webhooks`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                if (response.status !== 200) return cb(new Error(response.status + ':' + response.statusText));
                return response.json();
            }).then((hooks) => {
                return cb(null, hooks);
            }).catch((err) => {
                return cb(err);
            });
        },
        get: function(id, cb) {
            if (!id) return cb(new Error('Webhook ID required'));

            fetch(`${window.location.protocol}//${window.location.host}/api/webhooks/${id}`, {
                method: 'GET',
                credentials: 'same-origin'
            }).then((response) => {
                if (response.status !== 200) return cb(new Error(response.status + ':' + response.statusText));
                return response.json();
            }).then((hook) => {
                return cb(null, hook);
            }).catch((err) => {
                return cb(err);
            });
        }
    }
}
