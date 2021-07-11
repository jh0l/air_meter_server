#!/bin/bash
#copy next.js app build export to static folder with index.html going to /templates
cd web_app/air_meter_client
npm run build
npm run export
rm -R ../../static/*
cp -R out/* ../../static
cd ../../
rm library/templates/index.html
mv static/index.html library/templates/
