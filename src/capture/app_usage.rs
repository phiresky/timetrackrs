// import data from App Usage android app

/*
select datetime(act.time/1000, 'unixepoch'),
case act.type
when 262 then 'Screen off (locked)'
when 2565 then 'Screen on (unlocked)'
when 200 then 'Notification posted'
when 518 then 'Screen off'
when 2309 then 'Screen on (locked)'
when 1541 then 'Screen on (unlocked B)'
when 1285 then 'Screen on (locked)'
when 7 then 'Use'
else act.type end as type_str,
* from act
-- left join usage on usage.pid = act.pid
left join pkg on act.pid = pkg._id
where act.time > 1578873600000 and act.time < 1578956400000
order by time desc
*/
